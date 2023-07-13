import json

with open("colors.json") as f:
    colors = json.load(f)

with open("colors.txt") as f:
    color_names = [
        line.split('\t')
        for line in f.read().split('\n')
    ]

all_colors = []
all_colors_names = set()
all_colors_rgbs = set()

for rgb, _, name in color_names:
    r, g, b = rgb.split()
    
    if (r, g, b) in all_colors_rgbs:
        continue
    if ' ' in name:
        continue
    if name.startswith("X11"):
        continue
    if name.startswith("Web"):
        continue
    name = name.replace("Grey", "Gray")
    if name in all_colors_names:
        continue

    if name.lower() in [
     "black"    ,
     "red"      ,
     "green"    ,
     "yellow"   ,
     "blue"     ,
     "magenta"  ,
     "cyan"     ,
     "white"    ,
    ]:
        continue

    has_num = any(
        x.isdigit()
        for x in name
    )
    
    all_colors_names.add(name)
    all_colors.append((
        (int(r), int(g), int(b)),
        name,
        has_num
    ))

s = set()

all_colors.sort(key=lambda a: a[2])

print(all_colors)

new_colors_names = []

del colors[:16]

while True:
    best_dist = float("inf")
    best_index = None
    best = None

    for i, current_color in enumerate(colors):
        color = current_color["rgb"]
        cr, cg, cb = color["r"], color["g"], color["b"]

        for j, ((r, g, b), name, _has_digit) in enumerate(all_colors):
            dist = (r - cr) ** 2 + (g - cg) ** 2 + (b - cb) ** 2
            
            if best_dist > dist:
                best_dist = dist
                best_index = (i, j)

    if best_index is None:
        break
    
    print(best_dist)

    i, j = best_index
    
    better_name = all_colors[j][1]

    r, g, b, all_colors[j][0]

    better_name = better_name[0].upper() + better_name[1:]
    color = colors[i]
    
    test_color = f"\x1b[38;5;{color['colorId']}mtest\x1b[0m"
    test_color2 = f"\x1b[38;2;{r};{g};{b}mtest\x1b[0m"

    # if color["name"] == better_name:
    #     print(test_color, "*", color["name"])
    # else:
    #     print(test_color, color["name"], better_name)
    s.add(better_name)
    new_colors_names.append((
        test_color,
        best_dist,
        color,
        better_name,
    ))

    del all_colors[j]
    del colors[i]

new_colors_names.sort(key=lambda arg: arg[2]["colorId"])
matched = 0
prefix = 0

for test_color, test_color2, color, better_name in new_colors_names:
    print(color["colorId"], better_name)
    # color["name"] = color["name"].replace("Grey", "Gray")
    # if color["name"] == better_name:
    #     print(test_color, test_color2, "*", color["name"])
    #     matched += 1
    # else:
    #     if color["name"].startswith(better_name):
    #         prefix += 1
    #     print(test_color, test_color2, color["name"], better_name)
    

print(matched)
print(prefix)
print((matched + prefix) / 256)