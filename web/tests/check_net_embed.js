// node web/tests/check_net_embed.js web/net.js engine-rs/nets/td-best.net
// The weights embedded in web/net.js must be exactly the bytes of the net file.
const fs = require("fs");
const src = fs.readFileSync(process.argv[2], "utf8");
const m = src.match(/MAHBUSA_NET_B64\s*=\s*"([^"]+)"/);
if (!m) { console.error("no MAHBUSA_NET_B64 in", process.argv[2]); process.exit(1); }
const embedded = Buffer.from(m[1], "base64");
const file = fs.readFileSync(process.argv[3]);
const same = embedded.length === file.length && embedded.equals(file);
console.log(same ? `embedded net matches ${process.argv[3]} (${file.length} bytes)` : "EMBEDDED NET DIFFERS FROM THE NET FILE");
process.exit(same ? 0 : 1);
