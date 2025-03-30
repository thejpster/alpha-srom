# Unpacks one byte-wise file into eight bit-wise files.

from binascii import hexlify

f = open("am27c010_image_alphastation500.bin", "rb")
contents = f.read()
for idx in range(0, 8):
    count = 0
    acc = 0
    data = []
    # print(f"Extracting bit {idx} from {hexlify(contents)}")
    for b in contents:
        bit = (b >> idx) & 1
        # print(f"Byte {b:08b} Bit  {bit:01b}")
        acc >>= 1
        acc |= (bit << 7)
        count = count + 1
        if count == 8:
            # print(f"Acc  {acc:08b}")
            data.append(acc)
            acc = 0
            count = 0
    o = open(f"srom_{idx}.bin", "wb")
    o.truncate()
    data = bytes(data)
    # print(f"Writing {hexlify(data)}")
    o.write(data)
    o.close()
