from pathlib import Path

wasm_path = Path("target/wasm32-unknown-unknown/release/escrow.wasm")
out = Path("tmp/escrow_arg_hex.did")

wasm = wasm_path.read_bytes()

# Build Candid string with \hh escapes
escaped = ''.join(f'\\{b:02x}' for b in wasm)
out.write_text(f'(blob "{escaped}")\n', encoding="utf-8")
print(f"Wrote hex-escaped DID to {out} (size {out.stat().st_size})")
