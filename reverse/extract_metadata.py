#!/usr/bin/env python3
"""Extract global-metadata.dat from base.apk"""
import struct, zlib, os, sys

APK = "/home/z/my-project/repos/hlpatch/base.apk"
OUT = "/home/z/my-project/repos/hlpatch/reverse/global-metadata.dat"

print("Parsing base.apk local file headers...", flush=True)

entries = []
with open(APK, 'rb') as f:
    count = 0
    while count < 10000:
        sig = f.read(4)
        if sig != b'PK\x03\x04':
            f.seek(-3, 1)
            continue
        header = f.read(26)
        if len(header) < 26:
            break
        ver, flags, method, mtime, mdate, crc, comp_size, uncomp_size, fname_len, extra_len = struct.unpack('<HHHHHIIIHH', header)
        fname_bytes = f.read(fname_len)
        try:
            fname = fname_bytes.decode('utf-8')
        except:
            fname = fname_bytes.decode('latin-1')
        f.read(extra_len)
        data_offset = f.tell()
        entries.append({
            'filename': fname, 'method': method,
            'comp_size': comp_size, 'uncomp_size': uncomp_size,
            'data_offset': data_offset,
        })
        f.seek(data_offset + comp_size)
        count += 1

print(f"Total entries: {len(entries)}", flush=True)

# Find global-metadata.dat
target = None
for e in entries:
    if 'metadata' in e['filename'].lower() or 'global' in e['filename'].lower():
        print(f"  Match: {e['filename']} comp={e['comp_size']} uncomp={e['uncomp_size']}")
        target = e

if not target:
    # Search by extracting and checking magic
    print("Not found by name. Checking large files for magic bytes...", flush=True)
    MAGIC = b'\xAF\x1B\xB1\xFA'
    for e in entries:
        if e['uncomp_size'] > 1000000:
            with open(APK, 'rb') as f:
                f.seek(e['data_offset'])
                comp_data = f.read(e['comp_size'])
                if e['method'] == 8:
                    try:
                        data = zlib.decompress(comp_data, -15)
                    except:
                        continue
                else:
                    data = comp_data
                if data[:4] == MAGIC:
                    print(f"  FOUND: {e['filename']} ({len(data)} bytes)")
                    target = e
                    break

if target:
    print(f"Extracting {target['filename']}...", flush=True)
    with open(APK, 'rb') as f:
        f.seek(target['data_offset'])
        comp_data = f.read(target['comp_size'])
        if target['method'] == 8:
            data = zlib.decompress(comp_data, -15)
        else:
            data = comp_data
    with open(OUT, 'wb') as f:
        f.write(data)
    print(f"  Saved {len(data)} bytes to {OUT}")
    print(f"  Magic: {data[:4].hex()} ({'OK' if data[:4] == b'\xAF\x1B\xB1\xFA' else 'BAD'})")
else:
    print("global-metadata.dat not found in base.apk", flush=True)
    print("\nAll entries:", flush=True)
    for e in entries:
        print(f"  {e['filename']} comp={e['comp_size']} uncomp={e['uncomp_size']}", flush=True)
