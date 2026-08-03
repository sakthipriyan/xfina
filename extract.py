import pypdf
import sys

try:
    reader = pypdf.PdfReader(sys.argv[1])
    if reader.is_encrypted:
        # CAMS default password is pan (lowercase) + DOB (DDMMYYYY)?
        # Actually I can just try decrypting with the password if we know it.
        # But wait! I can just use the rust bin if I give it the password!
        pass
