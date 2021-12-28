TextEncoder.prototype.encode = v =>
  Uint8Array.from(v.split("").map(x => x.codePointAt(0)));
TextDecoder.prototype.decode = (v, _) =>
  Array.from(v||[]).map(x => String.fromCharCode(x)).join("");
function TextEncoder() {}
function TextDecoder() {}
