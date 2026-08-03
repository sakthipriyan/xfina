with open('web/src/App.vue', 'r') as f:
    lines = f.readlines()

def check_tags(start, end, label):
    depth = 0
    print(f"--- {label} ---")
    for i in range(start, end):
        line = lines[i]
        if '<div' in line: depth += line.count('<div')
        if '</div' in line: depth -= line.count('</div')
        print(f"{i+1}: [depth: {depth}] {line.rstrip()}")
    print(f"Final depth: {depth}\n")

check_tags(700, 866, "MF")
check_tags(1060, 1152, "Equity")
