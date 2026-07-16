# Phoenix LiveView vs. Gleam Lustre: Browser ⇄ Server State-Sync Protocols

A side-by-side reading of the actual source for the two frameworks. Phoenix
LiveView is checked out from `phoenixframework/phoenix_live_view`
(`lib/phoenix_live_view/{channel,diff,engine}.ex` on the server,
`assets/js/phoenix_live_view/{live_socket,view,rendered,constants}.ts` on the
client). Lustre is checked out from `lustre-labs/lustre`
(`src/lustre/runtime/transport.gleam`, `src/lustre/runtime/server/runtime.gleam`,
`src/lustre/vdom/{patch,vnode,vattr}.gleam`,
`priv/static/lustre-server-component.mjs`).

The goal is to compare *what flows over the wire* and *what model of "state"
each side actually owns*.

---

## 1. The shape of "state" each protocol assumes

### LiveView: state is an **iodata template**

LiveView never sends a virtual DOM. Templates compiled by
`Phoenix.LiveView.Engine` produce a `Phoenix.LiveView.Rendered` struct with
three fields (`engine.ex:108`):

```elixir
defstruct [:static, :dynamic, :fingerprint, :root, caller: :not_available]
```

* `static` is a list of literal string chunks the compiler extracted from the
  template — `["foo", "bar", "baz"]`.
* `dynamic` is a function `(track_changes? -> [dyn])`. Each `dyn` is iodata,
  another `Rendered`, a `Comprehension`, or a `Component`.
* `fingerprint` is a 128-bit hash of the *shape* of the template
  (`engine.ex:1338-1348`). Two renders with the same fingerprint are known to
  share the same `static` list and the same dynamic *slots*.

`Phoenix.LiveView.Diff.render/4` (`diff.ex:155-160`) walks the rendered tree
and produces a JSON-friendly **map of integer-keyed slots** that mirrors the
template shape. The key constants are defined in `diff.ex:15-23` and mirrored
in the JS client at `assets/js/phoenix_live_view/constants.ts:113-122`:

```
:s  static       (list of strings, or template index)
:d  / 0,1,2…     dynamic slot values, keyed by position
:r  root flag
:c  components
:e  events buffer
:t  title
:p  templates (shared static lists)
:k  keyed comprehension entries  + :kc count
:stream  stream patches
```

On a re-render LiveView walks the previous and the new `Rendered` recursively
(`diff.ex:400-487`). If a subtree's fingerprint matches the previous one, the
static list is *omitted* and only the changed dynamic slots are emitted — the
client patches them into its retained tree (`rendered.js:195-277:
mergeDiff` / `doMutableMerge`). Comprehensions are encoded as a shared static
plus a list of per-iteration dynamic maps (`:k`/`:kc`), so the static prose
between a list's items is sent once for the whole loop.

The client then turns the merged tree back into an HTML string
(`rendered.js:143-170: toString` / `recursiveToString`) and the *DOM
reconciler* in `dom_patch.ts` morphdom-style updates the real DOM. The diff
on the wire is therefore "what changed in the template", not "what changed
in the DOM".

### Lustre: state is a **virtual DOM**

Lustre's source of truth is an ordinary Elm-style VDOM tree
(`src/lustre/vdom/vnode.gleam:16-70`):

```gleam
pub type Element(message) {
  Fragment(...)
  Element(kind, key, namespace, tag, attributes, children, keyed_children,
          self_closing, void)
  Text(kind, key, content)
  UnsafeInnerHtml(...)
  Map(kind, key, mapper, child)
  Memo(kind, key, dependencies, view)
}
```

After every `update` the server runtime calls `view(model)` and runs a real
VDOM diff (`runtime.gleam:273-284`):

```gleam
EffectDispatchedMessage(message:) -> {
  let #(model, effect) = state.update(state.model, message)
  let vdom = state.view(model)
  let diff = diff(state.cache, state.vdom, vdom)
  ...
  let message = transport.reconcile(diff.patch, cache.memos(diff.cache))
  let _ = broadcast(state.subscribers, state.callbacks, message)
  ...
}
```

The wire payload is a tree of **`Patch` records**
(`src/lustre/vdom/patch.gleam:31-73`):

```gleam
pub type Patch(message) {
  Patch(index: Int, path: List(Int), removed: Int,
        changes: List(Change(message)),
        children: List(Patch(message)))
}

pub type Change(message) {
  ReplaceText(kind, content)
  ReplaceInnerHtml(kind, inner_html)
  Update(kind, added: List(Attribute), removed: List(Attribute))
  Move(kind, key, before)
  Replace(kind, index, with: Element)
  Remove(kind, index)
  Insert(kind, children, before)
}
```

Each `Patch` carries its position by `(index, path)` *into the live DOM* — the
client's reconciler walks that path and applies the listed changes
(`lustre-server-component.mjs:191-200`). Compare LiveView's "diff is a tree
parallel to the template" versus Lustre's "patch is an opcode list against the
DOM".

---

## 2. Mount / handshake

### LiveView

1. Server-rendered HTML carries a signed `data-phx-session` token (and
   optionally `data-phx-static`).
2. JS `LiveSocket` opens a *Phoenix Channel* (WebSocket or long-poll; see
   `live_socket.ts:334: new phxSocket(url, opts)` — it delegates to the
   `phoenix` JS package).
3. It joins a topic per view (`lv:<id>`), passing the session token as a join
   payload. `channel.ex:1067-1144` runs `mount/3` then `handle_params/3`,
   produces a full `Rendered`, and replies (`channel.ex:1400-1429`):
   ```elixir
   {:ok, %{rendered: diff, liveview_version: lv_vsn}}
   ```
   The reply contains the *full* initial diff (statics + dynamics). The
   client constructs `new Rendered(...)`, materializes HTML, and morphs into
   the DOM that was already there from the dead render.

### Lustre

1. The server runtime (`runtime.gleam:74-123`) is an OTP `actor` that owns
   `(model, vdom, cache)`. It is started independently of any transport.
2. A user-supplied HTTP/WS handler turns each new connection into a client
   subject that the runtime registers via `ClientRegisteredSubject` or
   `ClientRegisteredCallback` (`runtime.gleam:174-247`).
3. On registration the runtime immediately sends a `Mount` `ClientMessage`
   carrying the entire current VDOM plus the configuration the client needs
   to register itself as a custom element (`transport.gleam:13-30`):
   ```gleam
   Mount(kind: 0,
         open_shadow_root, will_adopt_styles,
         observed_attributes,        // host attrs we forward to server
         observed_properties,        // host JS properties we forward
         requested_contexts,         // DOM `context-request` keys
         provided_contexts,          // keys we provide downward
         vdom, memos)
   ```
4. The client custom element `<lustre-server-component>` connects via one of
   three transports based on the `method=` attribute
   (`lustre-server-component.mjs:901-911`):
   ```js
   case "ws":      this.#transport = new WebsocketTransport(...)
   case "sse":     this.#transport = new SseTransport(...)        // server → client only
   case "polling": this.#transport = new PollingTransport(...)    // GET-only
   ```
   The WebSocket transport is a *bare* WS to a route the user supplies —
   there is no Phoenix-channel framing.
5. On receiving `Mount`, the client attaches a shadow root, builds a
   `Reconciler`, registers JS property setters, requests any `context-request`
   keys, and calls `reconciler.mount(data.vdom)`
   (`lustre-server-component.mjs:721-816`). Unlike LiveView there is no
   already-rendered dead HTML — the shadow root starts empty and is filled
   from the mount payload.

The two designs diverge sharply here:

| Property              | LiveView                                | Lustre                                      |
|-----------------------|-----------------------------------------|---------------------------------------------|
| Initial HTML          | Server-rendered dead render            | None; client renders the `Mount` payload    |
| Identity / auth       | Signed session token in the HTML        | App's job; client just sends `csrf-token`   |
| Transport             | Phoenix.Socket (WS *or* long-poll)      | Bare WS, SSE, or polling — user picks       |
| Multi-client per view | One channel per view, parent/child tree | One runtime can have N subscribers (`runtime.gleam:174`) |
| Reconnect             | Phoenix Channel rejoin                  | `WebsocketTransport` exponential backoff (`...mjs:1010-1032`) |

---

## 3. Server → client messages

### LiveView events on the channel

`channel.ex:993-995` shows the entire downstream surface that flows over the
channel after mount:

```elixir
defp push_diff(state, diff, ref) when diff == %{}, do: push_noop(state, ref)
defp push_diff(state, diff, nil = _ref), do: push(state, "diff", diff)
defp push_diff(state, diff, ref),         do: reply(state, ref, :ok, %{diff: diff})
```

Plus a small handful of side-channel events (`channel.ex:971-995`):

* `"diff"` — diff against the retained `Rendered`. May contain a `:r` reply
  payload, `:e` `[event, payload]` push-events, `:t` page title, `:c`
  component updates, `:stream` patches.
* `"live_patch"` — same view, new URL/params; client patches the address bar.
* `"redirect"` / `"live_redirect"` — navigation.

That's almost the whole protocol from server to client. Every UI change is
expressed as one diff against the template structure.

### Lustre `ClientMessage`s

`transport.gleam:13-30` enumerates exactly six server-to-client kinds:

```
0  Mount          full vdom + handshake config
1  Reconcile      a Patch tree against the live vdom
2  Emit           {name, data} → fires a DOM CustomEvent on the host
3  Provide        publish a `context` value to descendants
4  Subscribe      subscribe to a host-provided context key
5  Unsubscribe    inverse of Subscribe
```

`Reconcile`'s payload is `patch.to_json(patch, memos)`
(`patch.gleam:151-163`), which produces objects like

```json
{"path":[2,0],"index":0,"removed":0,
 "changes":[{"kind":2,"added":[...],"removed":[...]}],
 "children":[...]}
```

where `kind` numbers map to `replace_text=0 / replace_inner_html=1 / update=2
/ move=3 / remove=4 / replace=5 / insert=6`. The client decodes them in
`lustre-server-component.mjs:818-820: case reconcile_kind`, which simply does
`this.#reconciler.push(data.patch)`.

`Memo` nodes carry referential-equality dependencies; the diff skips them when
deps are unchanged, and `memos` is sent alongside so the client can resolve
references on the receiving side (`runtime.gleam:280, 194`).

---

## 4. Client → server messages

### LiveView pushes

The client uses a single channel event name, `"event"`, with a tagged payload
(`view.ts:1620, 1791, 1875` etc.):

```js
this.pushWithReply(refGenerator, "event", {
  type: "click" | "keydown" | "form" | "hook" | "change" | ...,
  event: phxEvent,         // the phx-click / phx-submit value
  value: extractedMeta,    // or FormData URL-encoded
  cid: closestComponentID, // for LiveComponents
  uploads: ...,            // for file inputs
})
```

Server side, all of these are dispatched by one handler in
`channel.ex:246-263`:

```elixir
def handle_info(%Message{topic: topic, event: "event"} = msg, %{topic: topic} = state) do
  %{"value" => raw_val, "event" => event, "type" => type} = payload = msg.payload
  val = decode_event_type(type, raw_val, msg.payload)
  ...
  view_handle_event(...) | inner_component_handle_event(...)
end
```

The reply is *guaranteed*: `pushWithReply` is a Phoenix-channel push that
gets a `phx_reply` with a `ref`, and the server attaches the resulting diff
inline (`channel.ex:995: reply(state, ref, :ok, %{diff: diff})`). The same
`ref` mechanism is what carries `:r` reply payloads from
`handle_event/3 -> {:reply, %{}, socket}` back to the JS push callback.

Other client-to-server channel events:

* `"event"` (as above; `type` differentiates `click`, `keyup`, `form`,
  `change`, `hook`, …),
* `"live_patch"` (`channel.ex:138`) — back/forward navigation,
* `"cids_will_destroy"` / `"cids_destroyed"` (`channel.ex:156-243`) —
  client-side tombstoning of components,
* `"progress"` — upload progress (`channel.ex:170`),
* upload chunks travel on a *separate* `Phoenix.LiveView.UploadChannel`
  (`upload_channel.ex`).

### Lustre `ServerMessage`s

`transport.gleam:32-38` defines just five kinds:

```
0  AttributeChanged   name, value           ← host attribute mutated
1  EventFired         path, name, event     ← a DOM event was decoded server-side
2  PropertyChanged    name, value           ← JS property setter on the host
3  Batch              messages              ← coalesced messages
4  ContextProvided    key, value            ← context-request answered
```

`EventFired`'s `path` is the *DOM-relative path* of the firing element,
encoded as a tab/CR-separated string of indices or keys
(`lustre-server-component.mjs:155-167: getPath`). The runtime resolves it via
`cache.handle(state.cache, path, name, event)` (`runtime.gleam:396-407`).

Two interesting consequences of this design:

* **The server never names DOM events itself.** When the view emits
  `event.on("click", handler)` the *decoder* is shipped down inside the vdom
  as an `Event` attribute (`vattr.gleam:18-28`):
  ```gleam
  Event(kind:2, name, handler: Decoder(Handler(message)),
        include: List(String),                  // properties to copy from the DOM event
        prevent_default: EventBehaviour,        // Never | Possible | Always
        stop_propagation: EventBehaviour,
        debounce: Int, throttle: Int)
  ```
  The client decodes the DOM event, copies just `include` fields, applies
  `prevent_default`/`stop_propagation` according to the policy, and sends
  `EventFired{path, name, event}` back. The server looks up the decoder
  cached by `path+name`, runs it, and produces the user-domain `message`.
* **Bare attributes count as messages too.** Because `<lustre-server-component>`
  is just a custom element, a `MutationObserver`
  (`lustre-server-component.mjs:648-672`) forwards every observed attribute
  *mutation done by surrounding code* up to the runtime as
  `AttributeChanged`. That is the official way the host page passes data
  into the running component.

### Flow-control note

Lustre's WebSocket transport uses a strict ping-pong model:

```js
send(data) {
  if (waitingForResponse) { queue.push(data); return; }
  socket.send(JSON.stringify(data)); waitingForResponse = true;
}
onmessage = (...) => { ...; if (queue.length) socket.send({batch}); else waitingForResponse=false; }
```

(`lustre-server-component.mjs:1034-1042` and `986-1001`). The client only has
one message in flight; pending messages collapse into a single `Batch`. This
gives implicit backpressure and lossless ordering, at the cost of latency
when the network is slow.

LiveView relies on Phoenix Channel framing, which uses per-message refs and
allows multiple in-flight pushes; ordering and reliability are inherited
from the channel.

---

## 5. Granularity of the diff — concrete example

Consider rendering `<div class="counter">{@count}</div>` and updating `@count`
from `1` to `2`.

### LiveView wire trace

**Mount reply** (full diff):
```json
{"rendered":
  {"s":["<div class=\"counter\">", "</div>"], "0":"1"}}
```
The `"0"` slot is dynamic position 0; the `"s"` array is the literal segments
flanking the dynamic.

**On `@count` change** the server pushes a `"diff"` event:
```json
{"0":"2"}
```
The static prose, the wrapping div, and even the slot map are not resent —
the client merges `{"0":"2"}` into the retained tree and re-emits the HTML
(`rendered.js:259-277: doMutableMerge`).

### Lustre wire trace

**Mount** (full vdom):
```json
{"kind":0, "open_shadow_root":..., "vdom":
  {"kind":1, "tag":"div",
   "attributes":[{"kind":0,"name":"class","value":"counter"}],
   "children":[{"kind":2,"content":"1"}]}}
```

**On message** the diff produces a `Reconcile` with one nested patch that
touches the text child:
```json
{"kind":1, "patch":
  {"path":[0], "index":0, "removed":0,
   "changes":[{"kind":0,"content":"2"}],
   "children":[]}}
```
`kind:0` inside a `Change` is `ReplaceText` (`patch.gleam:86-90`). The client
reconciler walks `path=[0]` to the text node and calls `setData(node, "2")`
(`lustre-server-component.mjs:128`).

Both diffs are minimal in payload, but they describe two different things:
LiveView says "slot 0 of the same template now equals `"2"`", Lustre says
"the text node at path `[0]` now contains `"2"`". LiveView's payload is
shorter and slot-shaped; Lustre's is structurally identical to the DOM
mutation that follows.

---

## 6. Trade-offs the protocols bake in

* **Where the rendering happens.** LiveView essentially streams *template
  values* and reuses the HTML string–to–DOM morphing path on the client. The
  client never holds a structured representation of "what the page should
  be" — it just retains the slot map and a serialized HTML output. Lustre
  streams *DOM operations* against a structured virtual tree the client
  keeps in memory.
* **Diff cost / locality.** LiveView's diffing is essentially free: the
  Elixir compiler already factored the template into statics, dynamics, and
  fingerprints, so render-time work is "did this slot change?". Lustre runs
  a normal VDOM diff (`vdom/diff.gleam`) on every update; the work scales
  with the size of the rendered tree rather than the set of changed
  variables.
* **Compositional units.** LiveView's `LiveComponent`s are stateful and
  identified by a `:cid` integer that the diff carries through; the client
  patches components by id (`:c` map in the diff). Lustre's components are
  separate runtimes you mount as a custom element — composition happens at
  the *transport* level, not inside one diff.
* **Side channels.** LiveView ships file uploads, navigation, JS commands,
  flash, hooks, streams, and reply payloads all inside the same diff
  protocol (`:e`, `:t`, `:r`, `:stream`, `cids_destroyed`, etc.). Lustre's
  protocol is comparatively minimal — six client messages, five server
  messages — and defers things like uploads or routing to the host
  application; it adds `Emit` (server-defined `CustomEvent`s on the host
  element), `Provide`/`Subscribe` (context propagation), and
  attribute/property mirroring as its only non-DOM channels.
* **Coupling to the framework.** LiveView is married to Phoenix Channels and
  the Plug session token, which gives it long-poll fallback, auth, presence,
  and broadcasts for free. Lustre's transport is whatever HTTP endpoint you
  write; the runtime is a plain OTP actor with `register/deregister`
  semantics, and a single runtime can serve multiple browsers
  (`runtime.gleam:174-247`).
* **Initial paint.** LiveView always has a dead render to morph; the user
  sees content before the WS is up. A stock Lustre server component renders
  an empty shadow root until `Mount` arrives over the socket — pre-rendering
  has to be wired up separately.

---

## 7. Cheat-sheet

```
                       LiveView                 Lustre server component
─────────────────── ─────────────────────────── ───────────────────────────────
Wire format         JSON over Phoenix.Channel   JSON over WS / SSE / polling
Server state        compiled template + assigns Elm-style vdom + cache + memos
Diff unit           slot in template tree        opcode patch keyed by DOM path
Static reuse        statics elided after mount,  vdom nodes kept identical;
                    shared via :p templates      Memo() skipped when deps eq.
Comprehensions      :k/:kc with shared :s        regular vdom + keyed_children
Components          :c cid map in same diff      separate <lustre-...> runtime
Server→client kinds 1 ("diff") + nav + redirects 6 (Mount, Reconcile, Emit,
                                                  Provide, Subscribe, Unsubscribe)
Client→server kinds 1 ("event") + uploads + nav 5 (AttributeChanged,
                                                  EventFired, PropertyChanged,
                                                  Batch, ContextProvided)
Event handler       phx-click="name" → server    Decoder(Handler) carried in
                    looks up handle_event/3      vdom; client copies `include`
                                                 fields; path identifies node
Backpressure        Phoenix Channel refs         Client holds 1 in-flight,
                                                 batches the rest
Initial paint       dead-render HTML present     empty shadow root until Mount
Auth                signed session token         user-provided + CSRF query
Transport choice    WS or long-poll              WS, SSE, or polling
Per-runtime clients usually 1 (the joiner)       N subscribers per runtime
```

Both protocols share the same coarse idea — keep the model on the server,
diff the rendered output, send the patch — but they make opposite choices
about *what they diff*. LiveView diffs the **template** and lets the client
reassemble HTML; Lustre diffs the **virtual DOM** and ships DOM operations.
Everything downstream (initial render, components, batching, side channels)
follows from that one design point.

---

## Appendix: where Glimr's Loom fits

Glimr (`glimr-org/framework`) is a newer Gleam framework that bundles a
LiveView-style template runtime called **Loom**. Its wire format is much
closer to LiveView's than Lustre's is — it copies the *statics/dynamics*
protocol rather than the *virtual-DOM patch* protocol.

**State shape** (`src/glimr/loom/loom.gleam:60-72`):

```gleam
pub type LiveTree {
  LiveTree(statics: List(String), dynamics: List(Dynamic))
}

pub type Dynamic {
  DynString(String)        // leaf
  DynTree(LiveTree)        // nested (conditional / component)
  DynList(List(LiveTree))  // comprehension
}
```

Templates (`.loom.html`) compile at build time to generated Gleam modules
under `src/compiled/loom/` that produce `LiveTree` values — the same
compile-time factoring LiveView does with HEEx.

**Wire format** (`src/glimr/loom/runtime.gleam:618-625`, `456-462`):

```json
// initial tree
{"s": ["<div>","</div>"], "d": ["1"]}
// diff
{"0": "2"}
```

Nested subtrees serialize as `{"d": {...}}` (`runtime.gleam:503-511`) and
loops (`DynList`) are diffed per-item when lengths match
(`runtime.gleam:535-567`) — direct analogs of LiveView's `:k`/`:kc`
comprehensions. Branch flips are detected by comparing the raw statics lists
(`runtime.gleam:497, 544`); no fingerprint hash.

**Transport** (`src/glimr/loom/live.gleam`, `live_socket.gleam`): raw mist
WebSocket, not Phoenix Channels. Glimr multiplexes multiple live components
on one connection with its own three-message framing:

```
client → server:  {type:"join",  id, module, token}
                  {type:"event", id, handler, event,
                                 special_vars:{value,checked,key}}
                  {type:"leave", id}
server → client:  {type:"trees",    id, s:[...], d:[...]}   // initial
                  {type:"patch",    id, d:{...index diff}}
                  {type:"redirect", url}
```

The join `token` is `module:props_json` signed with `APP_KEY`
(`live.gleam:97-118`) — same idea as LiveView's signed session token. Each
joined component gets its own OTP actor holding `(props_json,
prev_tree_json)`; on every event the actor calls the generated
`handle_json → render_json → diff_tree_json` pipeline and sends `SendPatch`
(`live_socket.gleam:108-140`).

### How Loom compares to LiveView and Lustre

| Feature                              | LiveView          | Loom (Glimr)             | Lustre server component  |
|--------------------------------------|-------------------|--------------------------|--------------------------|
| Diff unit                            | template slot     | template slot            | DOM opcode (Patch/Change)|
| Statics / dynamics split             | `:s` / index keys | `s` / index keys         | (n/a — VDOM)             |
| Compile-time template factoring      | yes (HEEx)        | yes (Loom→Gleam)         | (n/a)                    |
| Template fingerprint hash            | yes               | **no** — compares statics| (n/a)                    |
| Shared template dedup (`:p`)         | yes               | no                       | Memo dedup by ref eq     |
| Stateful component refs (`:c` / cid) | yes               | no (inlined `DynTree`)   | separate runtime         |
| Keyed comprehensions / streams       | yes (`:k`, stream)| same-length zip only     | `keyed_children`         |
| Reply payload on event               | yes               | no                       | via `Emit`               |
| `live_patch` URL nav                 | yes               | no (`redirect` only)     | no                       |
| Upload channel                       | yes               | no                       | no                       |
| Transport                            | Phoenix Channel   | raw mist WS + own frames | WS / SSE / polling       |
| Dead-render before socket connects   | yes               | not in what I read       | no                       |
| Per-connection concurrency           | 1 channel/view    | N components multiplexed | N subscribers per runtime|

**Bottom line.** Loom is the closest thing in the Gleam ecosystem to
LiveView's actual protocol — not just its architecture. It ports the
statics/dynamics split, the index-keyed diff, the per-iteration loop
diffing, the compile-time template factoring, and the signed-token join
handshake. It is still an earlier-generation implementation: no
fingerprints, no shared-template pool, no component cids, no keyed streams,
no reply/nav/upload side-channels, and it rides raw mist WS instead of
Phoenix Channels. If you want LiveView's diff model without leaving Gleam,
Loom is where to look; if you want the full LiveView protocol, Elixir's
`lissome` package still wins by hosting a Lustre app inside a real
Phoenix.LiveView.
