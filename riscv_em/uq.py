import sys

line_set = set()

for line in sys.stdin:
    if line in line_set:
        continue
    else:
        print(line,end="")
        line_set.add(line)
