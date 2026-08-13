/*
** Copyright (c) 2026 The Fossil Chinese Localization Project
**
** This program is free software; you can redistribute it and/or
** modify it under the terms of the Simplified BSD License (also
** known as the "2-Clause License" or "FreeBSD License".)
**
** This program is distributed in the hope that it will be useful,
** but without any warranty; without even the implied warranty of
** merchantability or fitness for a particular purpose.
**
*******************************************************************************
**
** This file implements localization of the Fossil web interface.
**
** Fossil generates HTML by handing string literals to cgi_printf().  Those
** literals mix markup with the natural-language text that the user reads.
** Rather than fork every one of those literals, this module rewrites the
** natural language found inside them on the way out, using a translation
** catalog that is compiled into the executable as a builtin file named
** "locale/LOCALE.txt".
**
** The rewriting is deliberately conservative.  Text is replaced only when
** the whitespace-normalized source text matches a catalog entry exactly, so
** anything the catalog does not know about is emitted byte-for-byte as
** upstream wrote it.  A page is therefore never mangled by this module; at
** worst parts of it remain in English.
**
** Two further safety rules apply:
**
**    *  A translation is used only if it contains exactly the same sequence
**       of printf() conversions as the source text.  cgi_printf() formats are
**       being rewritten here, so a translation that dropped or reordered a
**       "%s" would read the wrong argument off the stack.
**
**    *  Text inside <script>, <style>, <pre> and <textarea> is never touched,
**       and neither is anything inside a markup tag except for the handful of
**       attributes that are known to hold prose (title=, placeholder=, alt=,
**       aria-label=, and value= on push buttons).
**
** Inline markup does not break a phrase.  In
**
**       @ Click <a href="%R/setup">here</a> to continue.
**
** the whole sentence is one translatable unit whose catalog key is
**
**       Click <1>here</1> to continue.
**
** The translation reuses <1>...</1> wherever Chinese word order needs it, and
** the original tag - URL, attributes and all - is substituted back in.  If a
** sentence like that has no translation, each of its plain-text fragments is
** looked up on its own, so partial catalogs still do some good.
**
** Set the "locale" setting to "en" to turn all of this off.
*/
#include "config.h"
#include <string.h>
#include "i18n.h"

/*
** The locale used when the "locale" setting has never been set.  This fork
** of Fossil ships a Chinese interface by default; "fossil setting locale en"
** restores the upstream English text.
*/
#define I18N_DFLT_LOCALE "zh-CN"

/*
** Maximum number of inline markup elements that may appear inside a single
** translatable phrase, and the largest number of translated format strings
** that are remembered between calls.
*/
#define I18N_MAX_PH     9
#define I18N_MAX_CACHE  5000

/*
** Values for the "kind" of a markup element.
*/
#define I18N_OPEN   1     /* <div>  */
#define I18N_CLOSE  2     /* </div> */
#define I18N_OTHER  3     /* <!-- comment -->, <!DOCTYPE ...>, <?xml ...?> */

/*
** Bit of the translator state that means "the previous chunk of output
** stopped in the middle of a tag".  The low bits of the state hold
** 1+index into azOpaqueElem[] when inside <script> and friends.
*/
#define I18N_ST_INTAG   0x100
#define I18N_ST_OPAQUE  0x0ff

/*
** One entry of a translation catalog.
*/
typedef struct I18nEntry I18nEntry;
struct I18nEntry {
  I18nEntry *pNext;      /* Next entry in the same collision chain */
  const char *zKey;      /* Whitespace-normalized source text */
  const char *zVal;      /* Text to show in place of zKey */
};

/*
** One memoized format-string translation.
*/
typedef struct I18nMemo I18nMemo;
struct I18nMemo {
  I18nMemo *pNext;       /* Next entry in the same collision chain */
  char *zIn;             /* An input string */
  char *zOut;            /* Its translation, or 0 if it needs none */
  int stIn;              /* Opaque-element state on entry */
  int stOut;             /* Opaque-element state on exit */
};

/*
** All state for this module.
*/
static struct {
  int isInit;            /* True after the first initialization attempt */
  int isFinal;           /* True when the choice of locale is settled */
  int isActive;          /* True if a catalog is loaded and non-empty */
  int bTrace;            /* True to log untranslated text to stderr */
  int stSkip;            /* 1+index into azOpaqueElem[] while inside <script>,
                         ** <style>, <pre> or <textarea>.  Fossil often opens
                         ** such an element in one cgi_printf() and closes it
                         ** in another, so this state outlives a single call */
  char *zLocale;         /* Name of the current locale.  Ex: "zh-CN" */
  char *zCatalog;        /* Catalog text.  Keys and values point into this */
  int nEntry;            /* Number of catalog entries */
  int nBucket;           /* Slots in aBucket[].  Always a power of two */
  I18nEntry **aBucket;   /* Catalog hash table */
  int nMemo;             /* Number of memoized translations */
  I18nMemo **aMemo;      /* Memo hash table, I18N_MAX_CACHE slots */
  I18nMemo **aMiss;      /* Text already reported by trace mode */
} i18n;

/*
** Markup that may appear in the middle of a phrase without ending it.
*/
static const char *const azInlineElem[] = {
  "a",    "abbr", "b",   "big",  "br",     "cite", "code", "em",
  "i",    "kbd",  "mark","q",    "s",      "samp", "small","span",
  "strong","sub", "sup", "tt",   "u",      "var",  "wbr",
};

/*
** Markup whose content is never natural language.
*/
static const char *const azOpaqueElem[] = {
  "pre", "script", "style", "textarea",
};

/*
** Inline markup with no closing tag.
*/
static const char *const azVoidElem[] = {
  "br", "wbr",
};

/*
** Attributes whose values are shown to the user as prose.
*/
static const char *const azTextAttr[] = {
  "alt", "aria-label", "placeholder", "title",
};

/*
** Return true if the nZ-byte name zZ occurs in az[], comparing without
** regard to case.
*/
static int i18n_name_in(
  const char *const *az,      /* Table of names to search */
  int nAz,                    /* Number of entries in az[] */
  const char *zZ, int nZ      /* The name to look for */
){
  int i;
  for(i=0; i<nAz; i++){
    if( (int)strlen(az[i])==nZ && fossil_strnicmp(az[i], zZ, nZ)==0 ) return 1;
  }
  return 0;
}

/*
** A 32-bit FNV-1a hash of nZ bytes of text.
*/
static unsigned int i18n_hash(const char *zZ, int nZ){
  unsigned int h = 2166136261U;
  int i;
  for(i=0; i<nZ; i++){
    h ^= (unsigned char)zZ[i];
    h *= 16777619U;
  }
  return h;
}

/*
** Append nZ bytes of zZ to pOut with every run of whitespace replaced by a
** single space and with leading and trailing whitespace omitted.
*/
static void i18n_normalize(Blob *pOut, const char *zZ, int nZ){
  int i = 0;
  int j = nZ;
  while( i<j && fossil_isspace(zZ[i]) ) i++;
  while( j>i && fossil_isspace(zZ[j-1]) ) j--;
  while( i<j ){
    if( fossil_isspace(zZ[i]) ){
      blob_append_char(pOut, ' ');
      while( i<j && fossil_isspace(zZ[i]) ) i++;
    }else{
      blob_append_char(pOut, zZ[i]);
      i++;
    }
  }
}

/*
** Normalize the nul-terminated string zZ in place, as i18n_normalize()
** would.  Return the new length.  Normalization never grows the text, so
** this is always safe.
*/
static int i18n_normalize_inplace(char *zZ){
  int i = 0;
  int j = 0;
  while( fossil_isspace(zZ[i]) ) i++;
  while( zZ[i] ){
    if( fossil_isspace(zZ[i]) ){
      while( fossil_isspace(zZ[i]) ) i++;
      if( zZ[i]==0 ) break;
      zZ[j++] = ' ';
    }else{
      zZ[j++] = zZ[i++];
    }
  }
  zZ[j] = 0;
  return j;
}

/*
** Append to pOut a summary of the printf() conversions in nZ bytes of zZ:
** one character per conversion, in order, plus a "*" for every conversion
** whose width or precision is taken from an argument.
**
** Two strings that produce the same summary consume the same arguments in
** the same order, which is what makes it safe to substitute one for the
** other in a cgi_printf() format.
*/
static void i18n_conv_summary(Blob *pOut, const char *zZ, int nZ){
  int i;
  for(i=0; i<nZ; i++){
    if( zZ[i]!='%' ) continue;
    i++;
    if( i>=nZ ) break;
    if( zZ[i]=='%' ) continue;
    while( i<nZ && (zZ[i]=='-' || zZ[i]=='+' || zZ[i]==' ' || zZ[i]=='#'
                 || zZ[i]=='!' || zZ[i]=='0' || zZ[i]==',') ){
      i++;
    }
    while( i<nZ && (fossil_isdigit(zZ[i]) || zZ[i]=='*') ){
      if( zZ[i]=='*' ) blob_append_char(pOut, '*');
      i++;
    }
    if( i<nZ && zZ[i]=='.' ){
      i++;
      while( i<nZ && (fossil_isdigit(zZ[i]) || zZ[i]=='*') ){
        if( zZ[i]=='*' ) blob_append_char(pOut, '*');
        i++;
      }
    }
    while( i<nZ && zZ[i]=='l' ) i++;
    if( i<nZ ) blob_append_char(pOut, zZ[i]);
  }
}

/*
** Return true if zA and zB consume the same printf() arguments.
*/
static int i18n_conv_compatible(
  const char *zA, int nA,
  const char *zB, int nB
){
  Blob a = empty_blob;
  Blob b = empty_blob;
  int rc;
  i18n_conv_summary(&a, zA, nA);
  i18n_conv_summary(&b, zB, nB);
  rc = blob_size(&a)==blob_size(&b)
        && memcmp(blob_buffer(&a), blob_buffer(&b), blob_size(&a))==0;
  blob_reset(&a);
  blob_reset(&b);
  return rc;
}

/*
** Add one entry to the catalog hash table.  zKey and zVal must remain valid
** for as long as the catalog is loaded; they normally point into
** i18n.zCatalog.
*/
static void i18n_add_entry(const char *zKey, const char *zVal){
  unsigned int h;
  I18nEntry *p;
  for(p=i18n.aBucket[i18n_hash(zKey,(int)strlen(zKey)) & (i18n.nBucket-1)];
      p;
      p=p->pNext){
    if( strcmp(p->zKey, zKey)==0 ){
      p->zVal = zVal;               /* A later entry wins */
      return;
    }
  }
  h = i18n_hash(zKey, (int)strlen(zKey)) & (i18n.nBucket-1);
  p = fossil_malloc(sizeof(*p));
  p->zKey = zKey;
  p->zVal = zVal;
  p->pNext = i18n.aBucket[h];
  i18n.aBucket[h] = p;
  i18n.nEntry++;
}

/*
** Look up nZ bytes of zZ, which must already be whitespace-normalized.
** Return the translation, or 0 if there is none.
*/
static const char *i18n_find(const char *zZ, int nZ){
  I18nEntry *p;
  if( !i18n.isActive || nZ<=0 ) return 0;
  for(p=i18n.aBucket[i18n_hash(zZ,nZ) & (i18n.nBucket-1)]; p; p=p->pNext){
    if( (int)strlen(p->zKey)==nZ && memcmp(p->zKey, zZ, nZ)==0 ) return p->zVal;
  }
  return 0;
}

/*
** Discard the catalog, the memo table and the trace table.
*/
static void i18n_clear(void){
  int i;
  for(i=0; i<i18n.nBucket; i++){
    I18nEntry *p = i18n.aBucket[i];
    while( p ){
      I18nEntry *pNext = p->pNext;
      fossil_free(p);
      p = pNext;
    }
  }
  fossil_free(i18n.aBucket);
  for(i=0; i18n.aMemo && i<I18N_MAX_CACHE; i++){
    I18nMemo *p = i18n.aMemo[i];
    while( p ){
      I18nMemo *pNext = p->pNext;
      fossil_free(p->zIn);
      fossil_free(p->zOut);
      fossil_free(p);
      p = pNext;
    }
  }
  fossil_free(i18n.aMemo);
  for(i=0; i18n.aMiss && i<I18N_MAX_CACHE; i++){
    I18nMemo *p = i18n.aMiss[i];
    while( p ){
      I18nMemo *pNext = p->pNext;
      fossil_free(p->zIn);
      fossil_free(p);
      p = pNext;
    }
  }
  fossil_free(i18n.aMiss);
  fossil_free(i18n.zCatalog);
  fossil_free(i18n.zLocale);
  memset(&i18n, 0, sizeof(i18n));
}

/*
** Parse the catalog text held in i18n.zCatalog and build the hash table.
**
** The format is a sequence of records, each of which is a "msgid" line
** giving the text as it appears in the source code followed by a "msgstr"
** line giving the replacement:
**
**       # Comments run to the end of the line
**       msgid   Branches
**       msgstr  分支
**
** Whitespace in a msgid is normalized before comparison, so a phrase that
** the source code wraps across several lines is written on one line here.
** A record whose msgstr is empty is a reminder that the phrase still needs
** to be translated; it is ignored.
*/
static void i18n_parse_catalog(void){
  char *zZ = i18n.zCatalog;
  char *zKey = 0;
  int nLine = 0;

  i18n.nBucket = 1024;
  i18n.aBucket = fossil_malloc_zero(sizeof(I18nEntry*)*i18n.nBucket);
  while( zZ && zZ[0] ){
    char *zLine = zZ;
    char *zEnd = strchr(zZ, '\n');
    if( zEnd ){
      *zEnd = 0;
      zZ = zEnd+1;
    }else{
      zZ = 0;
    }
    nLine++;
    while( fossil_isspace(zLine[0]) ) zLine++;
    if( zLine[0]==0 || zLine[0]=='#' ) continue;
    if( strncmp(zLine, "msgid", 5)==0 && fossil_isspace(zLine[5]) ){
      zKey = zLine+5;
      if( i18n_normalize_inplace(zKey)==0 ) zKey = 0;
    }else if( strncmp(zLine, "msgstr", 6)==0 && fossil_isspace(zLine[6]) ){
      char *zVal = zLine+6;
      if( i18n_normalize_inplace(zVal)>0 && zKey!=0 ){
        i18n_add_entry(zKey, zVal);
      }
      zKey = 0;
    }else{
      /* Written straight to stderr: fossil_warning() would route this
      ** into the HTML reply and re-enter the translator. */
      fprintf(stderr, "locale/%s.txt:%d: expected \"msgid\" or \"msgstr\"\n",
              i18n.zLocale, nLine);
    }
  }
  i18n.isActive = i18n.nEntry>0;
}

/*
** Load the catalog for the named locale.  Return true if a non-empty
** catalog was found.  A locale of "en", "none" or "off" means "use the
** text that is in the source code" and loads nothing.
*/
int i18n_set_locale(const char *zLocale){
  const unsigned char *zData;
  char *zFile;
  int nData = 0;

  i18n_clear();
  i18n.isInit = 1;
  i18n.isFinal = 1;
  if( zLocale==0 || zLocale[0]==0
   || fossil_stricmp(zLocale,"en")==0
   || fossil_stricmp(zLocale,"none")==0
   || fossil_stricmp(zLocale,"off")==0
  ){
    i18n.zLocale = fossil_strdup("en");
    return 0;
  }
  i18n.zLocale = fossil_strdup(zLocale);
  zFile = mprintf("locale/%s.txt", zLocale);
  zData = builtin_file(zFile, &nData);
  fossil_free(zFile);
  if( zData==0 ){
    fprintf(stderr, "no translation catalog for locale \"%s\"\n", zLocale);
    return 0;
  }
  i18n.zCatalog = fossil_strndup((const char*)zData, nData);
  i18n_parse_catalog();
  i18n.aMemo = fossil_malloc_zero(sizeof(I18nMemo*)*I18N_MAX_CACHE);
  return i18n.isActive;
}

/*
** Work out which locale to use and load its catalog.  Called automatically
** the first time a translation is needed.
**
** The locale comes from the FOSSIL_LOCALE environment variable if that is
** set, and otherwise from the "locale" setting.  The setting cannot be read
** before the repository is open, so if the first translation happens that
** early the decision is revisited once the repository becomes available.
*/
static void i18n_init(void){
  const char *zEnv;
  if( i18n.isInit && (i18n.isFinal || !g.repositoryOpen) ) return;
  zEnv = fossil_getenv("FOSSIL_LOCALE");
  if( zEnv && zEnv[0] ){
    i18n_set_locale(zEnv);
  }else if( g.repositoryOpen ){
    char *zLocale = db_get("locale", 0);
    i18n_set_locale(zLocale);
    fossil_free(zLocale);
  }else{
    i18n_set_locale(I18N_DFLT_LOCALE);
    i18n.isFinal = 0;      /* Ask the repository again once it is open */
  }
  i18n.bTrace = fossil_getenv("FOSSIL_LOCALE_TRACE")!=0;
}

/*
** Return true if web pages should be translated.
*/
int i18n_is_active(void){
  i18n_init();
  return i18n.isActive;
}

/*
** Return the name of the active locale as a BCP-47 language tag, for use as
** the "lang" attribute of <html>.  Never returns NULL.
*/
const char *i18n_locale(void){
  i18n_init();
  return i18n.zLocale ? i18n.zLocale : "en";
}

/*
** Report nZ bytes of zZ as text that has no translation.  Only used when
** FOSSIL_LOCALE_TRACE is set, and each distinct string is reported once.
*/
static void i18n_trace_miss(const char *zZ, int nZ){
  unsigned int h;
  I18nMemo *p;
  int i;
  int nAlpha = 0;
  if( !i18n.bTrace || nZ<=0 ) return;
  for(i=0; i<nZ; i++){
    /* Braces mean this is a fragment of script, CSS or a TH1 template that
    ** happens to sit outside any markup this module recognizes.  Such text
    ** is never worth translating, so keep it out of the report. */
    if( zZ[i]=='{' || zZ[i]=='}' ) return;
    if( fossil_isalpha(zZ[i]) ) nAlpha++;
  }
  if( nAlpha<2 ) return;
  if( strstr(zZ,"://")!=0 ) return;     /* A URL, not a phrase */
  if( i18n.aMiss==0 ){
    i18n.aMiss = fossil_malloc_zero(sizeof(I18nMemo*)*I18N_MAX_CACHE);
  }
  h = i18n_hash(zZ, nZ) % I18N_MAX_CACHE;
  for(p=i18n.aMiss[h]; p; p=p->pNext){
    if( (int)strlen(p->zIn)==nZ && memcmp(p->zIn, zZ, nZ)==0 ) return;
  }
  p = fossil_malloc(sizeof(*p));
  p->zIn = fossil_strndup(zZ, nZ);
  p->zOut = 0;
  p->pNext = i18n.aMiss[h];
  i18n.aMiss[h] = p;
  /* stderr, not fossil_print(): during a web request fossil_print() feeds
  ** cgi_printf(), which would come straight back into this module. */
  fprintf(stderr, "msgid  %s\nmsgstr\n", p->zIn);
}

/*
** Determine the extent and kind of the markup element that begins at z[0],
** which must be "<".  Return the number of bytes in the element, or 0 if
** z does not begin a complete, well-formed element.
*/
static int i18n_element(
  const char *zZ, int nZ,            /* Text to examine */
  int *pKind,                        /* Out: I18N_OPEN, _CLOSE or _OTHER */
  const char **pzName, int *pnName,  /* Out: the element name */
  int *pSelfClose                    /* Out: true for <br/> style elements */
){
  int i;
  int kind = I18N_OPEN;
  *pKind = I18N_OTHER;
  *pzName = 0;
  *pnName = 0;
  *pSelfClose = 0;
  if( nZ<2 || zZ[0]!='<' ) return 0;
  if( zZ[1]=='!' || zZ[1]=='?' ){
    if( nZ>=4 && zZ[1]=='!' && zZ[2]=='-' && zZ[3]=='-' ){
      for(i=4; i+2<nZ; i++){
        if( zZ[i]=='-' && zZ[i+1]=='-' && zZ[i+2]=='>' ) return i+3;
      }
      return nZ;
    }
    for(i=2; i<nZ && zZ[i]!='>'; i++){}
    return i<nZ ? i+1 : nZ;
  }
  i = 1;
  if( zZ[i]=='/' ){
    kind = I18N_CLOSE;
    i++;
  }
  if( i>=nZ || !fossil_isalpha(zZ[i]) ) return 0;
  *pzName = &zZ[i];
  while( i<nZ && (fossil_isalnum(zZ[i]) || zZ[i]=='-') ) i++;
  *pnName = (int)(&zZ[i] - *pzName);
  while( i<nZ && zZ[i]!='>' ){
    if( zZ[i]=='"' || zZ[i]=='\'' ){
      char q = zZ[i++];
      while( i<nZ && zZ[i]!=q ) i++;
      if( i>=nZ ) return 0;
    }
    i++;
  }
  if( i>=nZ ) return 0;
  if( zZ[i-1]=='/' ) *pSelfClose = 1;
  *pKind = kind;
  return i+1;
}

/*
** Return the total length of an opaque element - <script> and friends -
** whose opening tag occupies the first nTag bytes of zZ, including its
** content and its closing tag.
*/
static int i18n_opaque_extent(
  const char *zZ, int nZ,        /* The element and everything after it */
  int nTag,                      /* Length of the opening tag */
  const char *zName, int nName,  /* Name of the element */
  int *pClosed                   /* Out: true if the closing tag was seen */
){
  int i;
  *pClosed = 0;
  for(i=nTag; i+2+nName<=nZ; i++){
    if( zZ[i]=='<' && zZ[i+1]=='/'
     && fossil_strnicmp(&zZ[i+2], zName, nName)==0
    ){
      int j = i+2+nName;
      while( j<nZ && zZ[j]!='>' ) j++;
      *pClosed = 1;
      return j<nZ ? j+1 : nZ;
    }
  }
  return nZ;
}

/*
** Copy to pOut the text that closes out an opaque element that an earlier
** call left open.  *pState identifies the element; it is reset to zero once
** the closing tag is found.  Returns the number of bytes consumed.
*/
static int i18n_resume_opaque(Blob *pOut, const char *zZ, int nZ, int *pState){
  const char *zName = azOpaqueElem[(*pState & I18N_ST_OPAQUE) - 1];
  int nName = (int)strlen(zName);
  int i;
  for(i=0; i+2+nName<=nZ; i++){
    if( zZ[i]=='<' && zZ[i+1]=='/'
     && fossil_strnicmp(&zZ[i+2], zName, nName)==0
    ){
      int j = i+2+nName;
      while( j<nZ && zZ[j]!='>' ) j++;
      if( j<nZ ) j++;
      blob_append(pOut, zZ, j);
      *pState &= ~I18N_ST_OPAQUE;
      return j;
    }
  }
  blob_append(pOut, zZ, nZ);
  return nZ;
}

/*
** Step to the next attribute of a markup element.  On entry *pI is an
** offset into the nZ-byte element zZ.  Return true if an attribute was
** found, in which case *pI is advanced past it.
*/
static int i18n_next_attr(
  const char *zZ, int nZ, int *pI,
  const char **pzName, int *pnName,   /* Out: attribute name */
  const char **pzVal, int *pnVal,     /* Out: attribute value, if quoted */
  char *pQuote                        /* Out: the quote character, or 0 */
){
  int i = *pI;
  *pzVal = 0;
  *pnVal = 0;
  *pQuote = 0;
  while( i<nZ && !fossil_isalpha(zZ[i]) ){
    if( zZ[i]=='>' ) return 0;
    i++;
  }
  if( i>=nZ ) return 0;
  *pzName = &zZ[i];
  while( i<nZ && (fossil_isalnum(zZ[i]) || zZ[i]=='-' || zZ[i]=='_') ) i++;
  *pnName = (int)(&zZ[i] - *pzName);
  while( i<nZ && fossil_isspace(zZ[i]) ) i++;
  if( i<nZ && zZ[i]=='=' ){
    i++;
    while( i<nZ && fossil_isspace(zZ[i]) ) i++;
    if( i<nZ && (zZ[i]=='"' || zZ[i]=='\'') ){
      char q = zZ[i++];
      *pQuote = q;
      *pzVal = &zZ[i];
      while( i<nZ && zZ[i]!=q ) i++;
      *pnVal = (int)(&zZ[i] - *pzVal);
      if( i<nZ ) i++;
    }else{
      while( i<nZ && !fossil_isspace(zZ[i]) && zZ[i]!='>' ) i++;
    }
  }
  *pI = i;
  return 1;
}

/*
** Copy the nZ-byte markup element zZ to pOut, translating the values of any
** attributes that hold prose.  Return true if anything changed.
*/
static int i18n_emit_element(Blob *pOut, const char *zZ, int nZ){
  const char *zName;
  const char *zVal;
  int nName, nVal, i;
  char cQuote;
  int isButton = 0;
  int isInput = 0;
  int changed = 0;
  int iPrev;
  Blob out = empty_blob;

  i = 1;
  if( zZ[i]=='/' ) i++;
  while( i<nZ && (fossil_isalnum(zZ[i]) || zZ[i]=='-') ) i++;
  isInput = (i==6 && fossil_strnicmp(&zZ[1],"input",5)==0)
         || (i==7 && fossil_strnicmp(&zZ[1],"button",6)==0);
  if( isInput ){
    int j = i;
    while( i18n_next_attr(zZ, nZ, &j, &zName, &nName, &zVal, &nVal, &cQuote) ){
      if( nName==4 && fossil_strnicmp(zName,"type",4)==0 && zVal!=0 ){
        isButton = (nVal==6 && fossil_strnicmp(zVal,"submit",6)==0)
                || (nVal==6 && fossil_strnicmp(zVal,"button",6)==0)
                || (nVal==5 && fossil_strnicmp(zVal,"reset",5)==0);
      }
    }
  }
  iPrev = 0;      /* Everything up to the first replacement is copied as-is */
  while( i18n_next_attr(zZ, nZ, &i, &zName, &nName, &zVal, &nVal, &cQuote) ){
    const char *zTr = 0;
    if( zVal!=0 && nVal>0 && cQuote!=0
     && (i18n_name_in(azTextAttr, count(azTextAttr), zName, nName)
         || (isButton && nName==5 && fossil_strnicmp(zName,"value",5)==0))
    ){
      Blob key = empty_blob;
      i18n_normalize(&key, zVal, nVal);
      zTr = i18n_find(blob_buffer(&key), blob_size(&key));
      if( zTr==0 ){
        i18n_trace_miss(blob_buffer(&key), blob_size(&key));
      }else if( strchr(zTr, cQuote)!=0
             || !i18n_conv_compatible(zVal, nVal, zTr, (int)strlen(zTr)) ){
        zTr = 0;
      }
      blob_reset(&key);
    }
    if( zTr ){
      blob_append(&out, &zZ[iPrev], (int)(zVal - &zZ[iPrev]));
      blob_append(&out, zTr, -1);
      iPrev = (int)(zVal - zZ) + nVal;
      changed = 1;
    }
  }
  if( changed ){
    blob_append(&out, &zZ[iPrev], nZ-iPrev);
    blob_append(pOut, blob_buffer(&out), blob_size(&out));
  }else{
    blob_append(pOut, zZ, nZ);
  }
  blob_reset(&out);
  return changed;
}

/*
** One inline element absorbed into a translatable phrase.
*/
typedef struct I18nPh I18nPh;
struct I18nPh {
  const char *zOpen;  int nOpen;    /* The opening tag as written in the source */
  const char *zClose; int nClose;   /* The matching closing tag, if any */
  const char *zName;  int nName;    /* Element name */
};

/*
** Expand a translation into pOut, replacing <1>, </1> and <1/> markers with
** the markup they stand for.  Return false, having written nothing useful,
** if the translation refers to a marker that does not exist.
*/
static int i18n_expand(
  Blob *pOut,                  /* Write the expansion here */
  const char *zVal,            /* The translation */
  const I18nPh *aPh, int nPh   /* Markup absorbed from the source text */
){
  int i;
  for(i=0; zVal[i]; i++){
    int iPh, isClose = 0, isVoid = 0, j;
    if( zVal[i]!='<' ){
      blob_append_char(pOut, zVal[i]);
      continue;
    }
    j = i+1;
    if( zVal[j]=='/' ){
      isClose = 1;
      j++;
    }
    if( !fossil_isdigit(zVal[j]) ){
      blob_append_char(pOut, zVal[i]);
      continue;
    }
    iPh = zVal[j] - '1';
    j++;
    if( !isClose && zVal[j]=='/' ){
      isVoid = 1;
      j++;
    }
    if( zVal[j]!='>' ){
      blob_append_char(pOut, zVal[i]);
      continue;
    }
    if( iPh<0 || iPh>=nPh ) return 0;
    if( isClose ){
      if( aPh[iPh].zClose==0 ) return 0;
      blob_append(pOut, aPh[iPh].zClose, aPh[iPh].nClose);
    }else{
      if( isVoid != (aPh[iPh].zClose==0) ) return 0;
      blob_append(pOut, aPh[iPh].zOpen, aPh[iPh].nOpen);
    }
    i = j;
  }
  return 1;
}

static int i18n_translate_html(Blob*, const char*, int, int, int*);

/*
** Translate the run of text that begins at zIn[*pI].  Append the result,
** translated or not, to pOut and advance *pI past the run.  Return true if
** the text was changed.
**
** When bNoInline is false the run may absorb inline markup, in which case
** the catalog key uses <1>...</1> markers in its place.
*/
static int i18n_text_run(
  Blob *pOut,                 /* Append output here */
  const char *zIn, int nIn,   /* The complete text being translated */
  int *pI,                    /* In/out: offset of the run */
  int bNoInline               /* True if inline markup ends a run */
){
  int i = *pI;                /* First byte of the run */
  int k = i;                  /* Cursor */
  int i0, k1;                 /* The run without its surrounding whitespace */
  int nPh = 0;                /* Number of inline elements absorbed */
  int nStack = 0;             /* Depth of unclosed inline elements */
  int aStack[I18N_MAX_PH];
  I18nPh aPh[I18N_MAX_PH];
  int hasText = 0;            /* True if the run holds a letter */
  int changed = 0;
  int nKey;                   /* Length of the normalized catalog key */
  char *zKey;                 /* The normalized catalog key */
  const char *zVal;           /* Its translation, if there is one */
  Blob key = empty_blob;
  Blob out = empty_blob;

  while( k<nIn ){
    int kind, nName, selfClose, nElem;
    const char *zName;
    if( zIn[k]!='<' ){
      if( fossil_isalpha(zIn[k]) ) hasText = 1;
      blob_append_char(&key, zIn[k]);
      k++;
      continue;
    }
    nElem = i18n_element(&zIn[k], nIn-k, &kind, &zName, &nName, &selfClose);
    if( nElem==0 ){
      if( k+1<nIn && (fossil_isalpha(zIn[k+1]) || zIn[k+1]=='/'
                      || zIn[k+1]=='!' || zIn[k+1]=='?') ){
        break;                            /* Incomplete markup ends the run */
      }
      blob_append_char(&key, zIn[k]);     /* A "<" that is just text */
      k++;
      continue;
    }
    if( bNoInline || kind==I18N_OTHER ) break;
    if( !i18n_name_in(azInlineElem, count(azInlineElem), zName, nName) ) break;
    if( kind==I18N_OPEN ){
      if( nPh>=I18N_MAX_PH ) break;
      aPh[nPh].zOpen = &zIn[k];
      aPh[nPh].nOpen = nElem;
      aPh[nPh].zClose = 0;
      aPh[nPh].nClose = 0;
      aPh[nPh].zName = zName;
      aPh[nPh].nName = nName;
      if( selfClose || i18n_name_in(azVoidElem,count(azVoidElem),zName,nName) ){
        blob_appendf(&key, "<%d/>", nPh+1);
      }else{
        aStack[nStack++] = nPh;
        blob_appendf(&key, "<%d>", nPh+1);
      }
      nPh++;
    }else{
      int t;
      if( nStack==0 ) break;
      t = aStack[nStack-1];
      if( aPh[t].nName!=nName
       || fossil_strnicmp(aPh[t].zName, zName, nName)!=0 ) break;
      nStack--;
      aPh[t].zClose = &zIn[k];
      aPh[t].nClose = nElem;
      blob_appendf(&key, "</%d>", t+1);
    }
    k += nElem;
  }
  *pI = k;

  if( nStack>0 ){
    /* Inline markup was left open when the run ended.  Retranslate this
    ** stretch without absorbing inline markup. */
    int st = 0;
    blob_reset(&key);
    blob_reset(&out);
    return i18n_translate_html(pOut, &zIn[i], k-i, 1, &st);
  }
  i0 = i;
  while( i0<k && fossil_isspace(zIn[i0]) ) i0++;
  k1 = k;
  while( k1>i0 && fossil_isspace(zIn[k1-1]) ) k1--;
  zKey = blob_str(&key);
  nKey = i18n_normalize_inplace(zKey);
  zVal = hasText ? i18n_find(zKey, nKey) : 0;
  if( zVal!=0
   && i18n_expand(&out, zVal, aPh, nPh)
   && i18n_conv_compatible(&zIn[i0], k1-i0,
                           blob_buffer(&out), blob_size(&out))
  ){
    blob_append(pOut, &zIn[i], i0-i);
    blob_append(pOut, blob_buffer(&out), blob_size(&out));
    blob_append(pOut, &zIn[k1], k-k1);
    changed = 1;
  }else if( zVal==0 && nPh>0 ){
    /* No translation for the phrase as a whole.  Try its pieces. */
    int st = 0;
    changed = i18n_translate_html(pOut, &zIn[i], k-i, 1, &st);
  }else{
    if( hasText && zVal==0 ) i18n_trace_miss(zKey, nKey);
    blob_append(pOut, &zIn[i], k-i);
  }
  blob_reset(&key);
  blob_reset(&out);
  return changed;
}

/*
** Translate the nIn-byte HTML fragment zIn into pOut.  Return true if the
** output differs from the input.
*/
static int i18n_translate_html(
  Blob *pOut,                /* Append output here */
  const char *zIn, int nIn,  /* The HTML to translate */
  int bNoInline,             /* True if inline markup ends a phrase */
  int *pState                /* In/out: unclosed opaque element, or 0 */
){
  int i = 0;
  int changed = 0;
  if( *pState & I18N_ST_INTAG ){
    /* Finish the tag that the previous chunk of output left open.  Copy it
    ** through unchanged: a tag split across two calls is not one this
    ** module can safely rewrite. */
    char q = 0;
    while( i<nIn && (q || zIn[i]!='>') ){
      if( q ){
        if( zIn[i]==q ) q = 0;
      }else if( zIn[i]=='"' || zIn[i]=='\'' ){
        q = zIn[i];
      }
      i++;
    }
    if( i<nIn ){
      i++;
      *pState &= ~I18N_ST_INTAG;
    }
    blob_append(pOut, zIn, i);
  }
  if( *pState & I18N_ST_OPAQUE ) i += i18n_resume_opaque(pOut, &zIn[i], nIn-i,
                                                        pState);
  while( i<nIn ){
    int kind, nName, selfClose, nElem;
    const char *zName;
    if( zIn[i]=='<' ){
      nElem = i18n_element(&zIn[i], nIn-i, &kind, &zName, &nName, &selfClose);
      if( nElem==0 ){
        if( i+1<nIn && (fossil_isalpha(zIn[i+1]) || zIn[i+1]=='/'
                        || zIn[i+1]=='!' || zIn[i+1]=='?') ){
          blob_append(pOut, &zIn[i], nIn-i);   /* Incomplete markup */
          *pState |= I18N_ST_INTAG;
          break;
        }
      }else if( kind==I18N_OPEN && !selfClose
             && i18n_name_in(azOpaqueElem,count(azOpaqueElem),zName,nName) ){
        int closed, j, nSkip;
        nSkip = i18n_opaque_extent(&zIn[i], nIn-i, nElem, zName, nName,
                                   &closed);
        blob_append(pOut, &zIn[i], nSkip);
        i += nSkip;
        if( !closed ){
          for(j=0; j<count(azOpaqueElem); j++){
            if( i18n_name_in(&azOpaqueElem[j], 1, zName, nName) ){
              *pState |= j+1;   /* Resumes on the next call */
              break;
            }
          }
        }
        continue;
      }else if( kind==I18N_OTHER ){
        blob_append(pOut, &zIn[i], nElem);   /* Comment or declaration */
        i += nElem;
        continue;
      }else{
        changed |= i18n_emit_element(pOut, &zIn[i], nElem);
        i += nElem;
        continue;
      }
    }
    changed |= i18n_text_run(pOut, zIn, nIn, &i, bNoInline);
  }
  return changed;
}

/*
** Translate a cgi_printf() format string.  The return value is either
** zFormat itself, when there is nothing to translate, or a string owned by
** this module that stays valid until the process exits.
*/
const char *i18n_format(const char *zFormat){
  unsigned int h;
  int stIn, state, changed;
  I18nMemo *p;
  Blob out = empty_blob;

  if( zFormat==0 || zFormat[0]==0 ) return zFormat;
  if( !i18n_is_active() ) return zFormat;
  stIn = i18n.stSkip;
  h = i18n_hash(zFormat, (int)strlen(zFormat)) % I18N_MAX_CACHE;
  for(p=i18n.aMemo[h]; p; p=p->pNext){
    if( p->stIn==stIn && strcmp(p->zIn, zFormat)==0 ){
      i18n.stSkip = p->stOut;
      return p->zOut ? p->zOut : zFormat;
    }
  }
  state = stIn;
  changed = i18n_translate_html(&out, zFormat, (int)strlen(zFormat), 0,
                                &state);
  if( i18n.nMemo>=I18N_MAX_CACHE ){
    /* The memo table is full, which can only happen if format strings are
    ** being built at run time.  Bounded memory matters more here than a
    ** complete translation, so give up on this one. */
    i18n.stSkip = state;
    blob_reset(&out);
    return zFormat;
  }
  p = fossil_malloc(sizeof(*p));
  p->zIn = fossil_strdup(zFormat);
  p->zOut = changed ? fossil_strdup(blob_str(&out)) : 0;
  p->stIn = stIn;
  p->stOut = state;
  p->pNext = i18n.aMemo[h];
  i18n.aMemo[h] = p;
  i18n.nMemo++;
  i18n.stSkip = state;
  blob_reset(&out);
  return p->zOut ? p->zOut : zFormat;
}

/*
** Forget which markup element the reply is in the middle of.  Called at the
** start of every page.
*/
void i18n_new_page(void){
  i18n.stSkip = 0;
}

/*
** Translate the nIn-byte HTML fragment zIn.  Return a string that the caller
** must fossil_free(), or 0 if the fragment needs no translation.
**
** Used for output produced by TH1 - the skin header and footer - which does
** not pass through cgi_printf().
*/
char *i18n_fragment(const char *zIn, int nIn){
  Blob out = empty_blob;
  if( zIn==0 || nIn<=0 || !i18n_is_active() ) return 0;
  if( !i18n_translate_html(&out, zIn, nIn, 0, &i18n.stSkip) ){
    blob_reset(&out);
    return 0;
  }
  return blob_str(&out);
}

/*
** Translate a run of HTML that is passed to cgi_printf() as an argument
** rather than as part of the format.  The result is owned by this module.
*/
const char *i18n_markup(const char *zText){
  return i18n_format(zText);
}

/*
** Translate a short piece of plain text such as a page title or a menu
** label.  The return value is either zText itself or catalog-owned memory;
** either way the caller must not free it.
*/
const char *i18n_text(const char *zText){
  const char *zVal;
  Blob key = empty_blob;
  if( zText==0 || zText[0]==0 || !i18n_is_active() ) return zText;
  i18n_normalize(&key, zText, (int)strlen(zText));
  zVal = i18n_find(blob_buffer(&key), blob_size(&key));
  if( zVal!=0
   && !i18n_conv_compatible(zText, (int)strlen(zText), zVal,
                            (int)strlen(zVal)) ){
    zVal = 0;
  }
  if( zVal==0 ) i18n_trace_miss(blob_buffer(&key), blob_size(&key));
  blob_reset(&key);
  return zVal ? zVal : zText;
}

/*
** Append zElem to pList as one element of a TCL list, adding braces if the
** element would otherwise not survive the round trip.
*/
static void i18n_append_list_elem(Blob *pList, const char *zElem){
  int i;
  int needBrace = zElem[0]==0;
  for(i=0; zElem[i] && !needBrace; i++){
    if( fossil_isspace(zElem[i]) || zElem[i]=='{' || zElem[i]=='}'
     || zElem[i]=='"' || zElem[i]=='\\' || zElem[i]=='[' || zElem[i]==']'
     || zElem[i]=='$' || zElem[i]==';' ){
      needBrace = 1;
    }
  }
  if( blob_size(pList)>0 ) blob_append_char(pList, ' ');
  if( needBrace ){
    blob_append_char(pList, '{');
    blob_append(pList, zElem, -1);
    blob_append_char(pList, '}');
  }else{
    blob_append(pList, zElem, -1);
  }
}

/*
** Translate the labels of the main menu.  zMenu is a TCL list of
** (label, URL, capability-expression, flags) quadruples; only the labels
** are touched.  Returns zMenu itself if nothing changed, and otherwise a
** new string obtained from fossil_malloc().
*/
const char *i18n_mainmenu(const char *zMenu){
  char **azElem = 0;
  int *anElem = 0;
  int nElem = 0;
  int i;
  int changed = 0;
  Blob out = empty_blob;

  if( zMenu==0 || !i18n_is_active() ) return zMenu;
  Th_FossilInit(TH_INIT_NO_REPO);
  if( Th_SplitList(g.interp, zMenu, (int)strlen(zMenu),
                   &azElem, &anElem, &nElem)!=TH_OK ){
    Th_Free(g.interp, azElem);
    return zMenu;
  }
  for(i=0; i<nElem; i++){
    char *zElem = fossil_strndup(azElem[i], anElem[i]);
    const char *zUse = zElem;
    if( (i%4)==0 ){
      zUse = i18n_text(zElem);
      if( zUse!=zElem ) changed = 1;
    }
    i18n_append_list_elem(&out, zUse);
    fossil_free(zElem);
  }
  Th_Free(g.interp, azElem);
  if( !changed ){
    blob_reset(&out);
    return zMenu;
  }
  return blob_str(&out);
}

/*
** SETTING: locale width=10 default=zh-CN
** The VALUE of this setting is the language used by the web interface.
** Set it to "en" to use the original English text of the Fossil source
** code.  Any other value names a translation catalog that is built into
** the Fossil executable; run "fossil test-locale list" to see which
** catalogs are available.
**
** This fork of Fossil defaults to "zh-CN", Simplified Chinese.
**
** The FOSSIL_LOCALE environment variable, when set, overrides this
** setting.  Setting FOSSIL_LOCALE_TRACE causes every phrase that has no
** translation to be printed in catalog format, which is a convenient way
** to find out what a translation is still missing.
*/

/*
** COMMAND: test-locale
**
** Usage: %fossil test-locale SUBCOMMAND ...
**
** Inspect and check the translation catalogs that are built into this
** executable.
**
** > fossil test-locale list
**
**        List the locales for which a catalog is available.
**
** > fossil test-locale check ?LOCALE?
**
**        Report catalog entries that would be rejected at run time
**        because the translation does not use the same printf()
**        conversions as the original.  Exits with a non-zero status if
**        any problem is found.
**
** > fossil test-locale dump ?LOCALE?
**
**        Print every entry of a catalog.
**
** > fossil test-locale tr TEXT ...
**
**        Translate each TEXT argument as if it were a cgi_printf()
**        format string and print the result.
**
** The LOCALE defaults to the value of the "locale" setting.
*/
void test_locale_cmd(void){
  const char *zCmd;
  int i;
  int nErr = 0;

  db_find_and_open_repository(OPEN_OK_NOT_FOUND|OPEN_SUBSTITUTE, 0);
  if( g.argc<3 ) usage("SUBCOMMAND ...");
  zCmd = g.argv[2];
  if( strncmp(zCmd, "list", strlen(zCmd))==0 ){
    int n = builtin_file_count();
    for(i=0; i<n; i++){
      const char *zName = builtin_file_name(i);
      int nName = (int)strlen(zName);
      if( strncmp(zName,"locale/",7)==0 && nName>11
       && strcmp(&zName[nName-4],".txt")==0 ){
        fossil_print("%.*s\n", nName-11, &zName[7]);
      }
    }
    return;
  }
  if( g.argc>3 && strncmp(zCmd,"tr",strlen(zCmd))!=0 ){
    i18n_set_locale(g.argv[3]);
  }else{
    i18n_is_active();
  }
  if( strncmp(zCmd, "check", strlen(zCmd))==0
   || strncmp(zCmd, "dump", strlen(zCmd))==0
  ){
    int bDump = zCmd[0]=='d';
    int nEntry = 0;
    if( !i18n.isActive ){
      fossil_fatal("no catalog loaded for locale \"%s\"", i18n_locale());
    }
    for(i=0; i<i18n.nBucket; i++){
      I18nEntry *p;
      for(p=i18n.aBucket[i]; p; p=p->pNext){
        nEntry++;
        if( bDump ){
          fossil_print("msgid  %s\nmsgstr %s\n", p->zKey, p->zVal);
        }
        if( !i18n_conv_compatible(p->zKey, (int)strlen(p->zKey),
                                  p->zVal, (int)strlen(p->zVal)) ){
          fossil_print("ERROR: conversions differ:\n"
                       "  msgid  %s\n  msgstr %s\n", p->zKey, p->zVal);
          nErr++;
        }
      }
    }
    if( !bDump ){
      fossil_print("locale %s: %d entries, %d problem%s\n",
                   i18n_locale(), nEntry, nErr, nErr==1 ? "" : "s");
    }
    if( nErr ) fossil_fatal("%d catalog problem(s)", nErr);
    return;
  }
  if( strncmp(zCmd, "tr", strlen(zCmd))==0 ){
    for(i=3; i<g.argc; i++){
      i18n_new_page();
      fossil_print("%s\n", i18n_format(g.argv[i]));
    }
    return;
  }
  fossil_fatal("unknown subcommand \"%s\": expected one of "
               "check dump list tr", zCmd);
}
