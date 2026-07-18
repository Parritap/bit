import hashlib
import zlib

content = b"hola"
print(f"content: {content}")

# 1. Create the HEADER: type + space + size + null byte (\0)
header = f"blob {len(content)}\0".encode()
print(f"header: {header}")

# 2. Combine the HEADER and the CONTENT
store = header + content
print(f"store: {store}")

# 3. Calculate the SHA-1 hash (This becomes the directory and filename)
sha1 = hashlib.sha1(store).hexdigest()
print(f"SHA-1: {sha1}")
# Output: b8b4a4e2a5db3ebed5f5e02beb3e2d27bca9fc9a

# 4. Compress the stored data (This is what actually gets written to disk)
zlib_content = zlib.compress(store)
print(zlib_content)

# (Optional) Writing it to a file exactly as Git would
import os

path = f".git/objects/{sha1[:2]}/{sha1[2:]}"
os.makedirs(os.path.dirname(path), exist_ok=True)
with open(path, "wb") as f:
    f.write(zlib_content)
