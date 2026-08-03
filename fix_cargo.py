import os
import glob

for path in glob.glob('**/Cargo.toml', recursive=True):
    if "node_modules" in path or "target" in path or path == "Cargo.toml" or path == "cli/Cargo.toml":
        continue
    with open(path, 'r') as f:
        lines = f.readlines()
    
    with open(path, 'w') as f:
        for line in lines:
            f.write(line)
            if "edition.workspace = true" in line:
                f.write('description.workspace = true\nlicense.workspace = true\nrepository.workspace = true\n')
print("Done")
