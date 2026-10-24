// Pull named top-level functions out of backrooms.js together with every
// top-level declaration they (transitively) reference, so a placer can run in
// node without the rest of the game.
//
// A small JS lexer splits the file into top-level statements (the file is
// consistently formatted: a new top-level statement starts at column 0), finds
// the names each statement declares and the free identifiers it uses.

const REGEX_AFTER_KW = new Set(['return', 'typeof', 'instanceof', 'in', 'of', 'new', 'delete', 'void', 'throw', 'case', 'do', 'else', 'yield', 'await']);
const PUNCT = ['>>>=', '...', '===', '!==', '**=', '<<=', '>>=', '>>>', '&&=', '||=', '??=', '=>', '==', '!=', '<=', '>=', '&&', '||', '??', '?.', '++', '--', '+=', '-=', '*=', '/=', '%=', '&=', '|=', '^=', '**', '<<', '>>'];
const isIdStart = (c) => (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || c === '_' || c === '$' || c > '\x7f';
const isIdPart = (c) => isIdStart(c) || (c >= '0' && c <= '9');
const isDigit = (c) => c >= '0' && c <= '9';

export function lex(src) {
  const toks = [];
  const n = src.length;
  let i = 0, line = 1, lineStart = 0, depth = 0;
  const stack = [];       // '{' '(' '[' or 't' (template substitution)
  let prev = null;
  const push = (type, value, start, d = depth) => {
    const t = { type, value, start, end: i, line, col: start - lineStart, depth: d };
    toks.push(t); prev = t; return t;
  };
  const regexOk = () => {
    if (!prev) return true;
    if (prev.type === 'num' || prev.type === 'str' || prev.type === 'tmpl' || prev.type === 're') return false;
    if (prev.type === 'id') return REGEX_AFTER_KW.has(prev.value);
    return !(prev.value === ')' || prev.value === ']' || prev.value === '}');
  };
  // read template characters; i is just after '`' or after the '}' that closed a substitution
  const readTemplate = () => {
    while (i < n) {
      const c = src[i];
      if (c === '\\') { i += 2; continue; }
      if (c === '`') { i++; return 'end'; }
      if (c === '$' && src[i + 1] === '{') { i += 2; return 'sub'; }
      if (c === '\n') { line++; lineStart = i + 1; }
      i++;
    }
    return 'end';
  };
  while (i < n) {
    const c = src[i];
    if (c === '\n') { line++; i++; lineStart = i; continue; }
    if (c === ' ' || c === '\t' || c === '\r' || c === '\f' || c === '\v' || c === ' ' || c === '﻿') { i++; continue; }
    if (c === '/' && src[i + 1] === '/') { while (i < n && src[i] !== '\n') i++; continue; }
    if (c === '/' && src[i + 1] === '*') {
      i += 2;
      while (i < n && !(src[i] === '*' && src[i + 1] === '/')) { if (src[i] === '\n') { line++; lineStart = i + 1; } i++; }
      i += 2; continue;
    }
    const start = i;
    if (isIdStart(c)) { while (i < n && isIdPart(src[i])) i++; push('id', src.slice(start, i), start); continue; }
    if (isDigit(c) || (c === '.' && isDigit(src[i + 1] || ''))) {
      while (i < n) {
        const d = src[i];
        if ((d === '+' || d === '-') && (src[i - 1] === 'e' || src[i - 1] === 'E') && !/^0[xX]/.test(src.slice(start, i))) { i++; continue; }
        if (isIdPart(d) || d === '.') { i++; continue; }
        break;
      }
      push('num', src.slice(start, i), start); continue;
    }
    if (c === '"' || c === "'") {
      i++;
      while (i < n && src[i] !== c) { if (src[i] === '\\') i++; i++; }
      i++; push('str', src.slice(start, i), start); continue;
    }
    if (c === '`') {
      i++;
      const r = readTemplate();
      push('tmpl', '`', start);
      if (r === 'sub') { stack.push('t'); depth++; }
      continue;
    }
    if (c === '/' && regexOk()) {
      i++;
      let inClass = false;
      while (i < n) {
        const d = src[i];
        if (d === '\\') { i += 2; continue; }
        if (d === '[') inClass = true; else if (d === ']') inClass = false;
        else if (d === '/' && !inClass) break;
        else if (d === '\n') break;
        i++;
      }
      i++;
      while (i < n && isIdPart(src[i])) i++;
      push('re', src.slice(start, i), start); continue;
    }
    if (c === '{' || c === '(' || c === '[') { i++; push('p', c, start); stack.push(c); depth++; continue; }
    if (c === '}' || c === ')' || c === ']') {
      i++;
      const top = stack.pop();
      depth--;
      if (top === 't' && c === '}') {
        const r = readTemplate();
        push('tmpl', '}`', start);
        if (r === 'sub') { stack.push('t'); depth++; }
        continue;
      }
      push('p', c, start); continue;
    }
    let p = null;
    for (const q of PUNCT) if (src.startsWith(q, i)) { p = q; break; }
    if (p === '?.' && isDigit(src[i + 2] || '')) p = null;
    if (!p) p = c;
    i += p.length;
    push('p', p, start);
  }
  return toks;
}

/** Split into top-level statements: { start, end, kind, names[], refs:Set, first } */
export function topLevel(src) {
  const toks = lex(src);
  const stmts = [];
  let cur = null;
  const close = (endTok) => { if (cur) { cur.endTok = endTok; stmts.push(cur); cur = null; } };
  for (let k = 0; k < toks.length; k++) {
    const t = toks[k];
    if (cur && t.depth === 0 && t.col === 0 && (t.type === 'id' || t.type === 'tmpl' || t.type === 'str' || (t.type === 'p' && (t.value === '(' || t.value === '[' || t.value === '{')))
        && toks[k - 1].line < t.line && !(t.type === 'id' && (t.value === 'else' || t.value === 'catch' || t.value === 'finally'))) close(k - 1);
    if (!cur) {
      if (t.type === 'p' && t.value === ';') continue;
      cur = { startTok: k, endTok: k };
    }
    if (t.depth === 0 && t.type === 'p' && t.value === ';') { close(k); continue; }
    const s0 = toks[cur.startTok];
    const isDeclFn = s0.value === 'function' || s0.value === 'class' || (s0.value === 'async' && toks[cur.startTok + 1].value === 'function');
    if (isDeclFn && t.depth === 0 && t.type === 'p' && t.value === '}') close(k);
  }
  close(toks.length - 1);
  for (const s of stmts) analyse(s, toks, src);
  return { stmts, toks };
}

function analyse(s, toks, src) {
  const a = s.startTok, b = s.endTok;
  s.start = toks[a].start; s.end = toks[b].end;
  s.line = toks[a].line;
  s.names = [];
  const t0 = toks[a];
  if (t0.value === 'function' || (t0.value === 'async' && toks[a + 1].value === 'function')) {
    let k = t0.value === 'async' ? a + 2 : a + 1;
    if (toks[k].value === '*') k++;
    s.kind = 'function'; s.names.push(toks[k].value);
  } else if (t0.value === 'class') {
    s.kind = 'class'; s.names.push(toks[a + 1].value);
  } else if (t0.value === 'const' || t0.value === 'let' || t0.value === 'var') {
    s.kind = t0.value;
    let k = a + 1;
    while (k <= b) {
      const t = toks[k];
      if (t.type === 'id') s.names.push(t.value);
      else if (t.value === '{' || t.value === '[') {
        // destructuring pattern: identifiers not followed by ':'
        const d = t.depth; k++;
        while (k <= b && !(toks[k].depth === d && (toks[k].value === '}' || toks[k].value === ']'))) {
          const u = toks[k];
          if (u.type === 'id' && toks[k + 1] && toks[k + 1].value !== ':' && toks[k - 1].value !== '=') s.names.push(u.value);
          k++;
        }
      }
      // skip to the next declarator
      k++;
      while (k <= b && !(toks[k].depth === 0 && toks[k].value === ',')) k++;
      k++;
    }
  } else {
    s.kind = 'stmt';
    s.first = t0.type === 'id' ? t0.value : null;
  }
  // names bound anywhere inside the statement (params, local const/let/var,
  // inner functions, catch params). Block scoping is ignored: a name bound
  // anywhere inside counts as local everywhere inside.
  const local = new Set();
  const own = new Set(s.names);
  const bindPattern = (k, stop) => {
    // identifiers of a binding pattern starting at token k; returns the index after it
    const t = toks[k];
    if (!t) return k + 1;
    if (t.type === 'id') { local.add(t.value); return k + 1; }
    if (t.value === '{' || t.value === '[') {
      const d = t.depth; k++;
      while (k <= stop && !(toks[k].depth === d && (toks[k].value === '}' || toks[k].value === ']'))) {
        const u = toks[k], nx = toks[k + 1], pv = toks[k - 1];
        if (u.type === 'id' && nx && nx.value !== ':' && pv && (pv.value === '{' || pv.value === '[' || pv.value === ',' || pv.value === ':' || pv.value === '...')) local.add(u.value);
        k++;
      }
      return k + 1;
    }
    return k + 1;
  };
  const params = (open) => {
    // '(' at token index `open`: collect parameter names
    const d = toks[open].depth;
    let k = open + 1;
    while (k <= b && !(toks[k].depth === d && toks[k].value === ')')) {
      const u = toks[k], pv = toks[k - 1], nx = toks[k + 1];
      if (u.depth === d + 1 && (u.value === '{' || u.value === '[') && (pv.value === '(' || pv.value === ',')) { k = bindPattern(k, b); continue; }
      if (u.type === 'id' && u.depth === d + 1 && (pv.value === '(' || pv.value === ',' || pv.value === '...') && nx && (nx.value === ',' || nx.value === ')' || nx.value === '=')) local.add(u.value);
      k++;
    }
    return k;
  };
  for (let k = a; k <= b; k++) {
    const t = toks[k];
    if (t.type === 'id' && (t.value === 'const' || t.value === 'let' || t.value === 'var') && !(k === a)) {
      // declarators until the statement's end at this depth
      let j = k + 1;
      for (;;) {
        j = bindPattern(j, b);
        const d = toks[k].depth;
        // skip initializer to a ',' at this depth (next declarator) or anything ending the declaration
        let par = 0;
        while (j <= b) {
          const u = toks[j];
          if (u.depth < d) break;
          if (u.depth === d && (u.value === ';' || u.value === ')' || u.value === '}' || u.value === 'of' || u.value === 'in')) break;
          if (u.depth === d && u.value === ',') break;
          // a newline-separated next statement at the same depth (ASI)
          if (u.depth === d && u.type === 'id' && toks[j - 1].line < u.line && /^(const|let|var|if|for|while|return|function|switch|try|do|break|continue|throw)$/.test(u.value)) break;
          j++;
        }
        if (j <= b && toks[j].depth === d && toks[j].value === ',') { j++; continue; }
        break;
      }
    } else if (t.type === 'id' && t.value === 'function') {
      let j = k + 1;
      if (toks[j] && toks[j].value === '*') j++;
      if (toks[j] && toks[j].type === 'id') { if (k !== a && !(t0.value === 'async' && k === a + 1)) local.add(toks[j].value); j++; }
      if (toks[j] && toks[j].value === '(') params(j);
    } else if (t.type === 'id' && t.value === 'catch' && toks[k + 1] && toks[k + 1].value === '(') {
      bindPattern(k + 2, b);
    } else if (t.type === 'id' && t.value === 'class' && k !== a && toks[k + 1] && toks[k + 1].type === 'id') {
      local.add(toks[k + 1].value);
    } else if (t.type === 'p' && t.value === '=>') {
      const p = toks[k - 1];
      if (p.type === 'id') local.add(p.value);
      else if (p.value === ')') {
        let j = k - 2; const d = p.depth;
        while (j > a && !(toks[j].depth === d && toks[j].value === '(')) j--;
        params(j);
      }
    } else if (t.type === 'id' && toks[k + 1] && toks[k + 1].value === '(' && toks[k - 1] && (toks[k - 1].value === '{' || toks[k - 1].value === ',' || toks[k - 1].value === '}' || toks[k - 1].value === ';')) {
      // object/class method shorthand `name(args) { ... }`: name is a key, args are params
      const d = toks[k + 1].depth;
      let j = k + 2;
      while (j <= b && !(toks[j].depth === d && toks[j].value === ')')) j++;
      if (toks[j + 1] && toks[j + 1].value === '{') { params(k + 1); t.methodKey = true; }
    }
  }
  s.refs = new Set();
  for (let k = a; k <= b; k++) {
    const t = toks[k];
    if (t.type !== 'id' || t.methodKey) continue;
    const p = toks[k - 1], q = toks[k + 1];
    if (p && (p.value === '.' || p.value === '?.')) continue;
    if (p && (p.value === '{' || p.value === ',') && q && q.value === ':') continue;   // object key
    if (local.has(t.value) && !own.has(t.value)) continue;
    s.refs.add(t.value);
  }
  s.locals = local;
  s.text = src.slice(s.start, s.end);
}

export class Bundle {
  constructor(src) {
    this.src = src;
    const { stmts } = topLevel(src);
    this.stmts = stmts;
    this.decl = new Map();      // name -> statement index (functions: the last declaration wins)
    this.attach = new Map();    // name -> [statement index] of `name.x = ...` / `name.f(...)` statements
    stmts.forEach((s, i) => {
      for (const nm of s.names) { if (s.kind === 'function' || !this.decl.has(nm)) this.decl.set(nm, i); }
      if (s.kind === 'stmt' && s.first) {
        if (!this.attach.has(s.first)) this.attach.set(s.first, []);
        this.attach.get(s.first).push(i);
      }
    });
  }
  /** statement indices needed by `roots`, minus anything declared in `exclude` */
  collect(roots, exclude, withAttachments = true) {
    const need = new Set();
    const names = new Set();
    const work = [...roots];
    while (work.length) {
      const nm = work.pop();
      if (names.has(nm) || exclude.has(nm)) continue;
      names.add(nm);
      const i = this.decl.get(nm);
      if (i === undefined) continue;
      const add = (j) => {
        if (need.has(j)) return;
        need.add(j);
        for (const r of this.stmts[j].refs) if (!names.has(r) && !exclude.has(r) && this.decl.has(r)) work.push(r);
      };
      add(i);
      if (withAttachments) for (const j of this.attach.get(nm) || []) add(j);
    }
    return { indices: [...need].sort((x, y) => x - y), names };
  }
  /** statements as [{text, kind, names, line}] in evaluation order: functions first, then the rest in source order */
  emit(indices) {
    const s = indices.map((i) => this.stmts[i]);
    return [...s.filter((x) => x.kind === 'function'), ...s.filter((x) => x.kind !== 'function')];
  }
}
