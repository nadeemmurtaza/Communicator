import glob, re
from collections import Counter

emoji_pattern = re.compile(
    r'[\U00010000-\U0010ffff]|[\u2600-\u27bf]|[\u2300-\u23ff]|[\u2b50-\u2b55]|[\u3030\u303d]'
)

files = glob.glob('frontend/src/**/*.tsx', recursive=True) + glob.glob('frontend/src/**/*.ts', recursive=True)

file_counts = Counter()
lines_by_file = {}

with open('emoji_audit.txt', 'w', encoding='utf-8') as out:
    for path in sorted(files):
        if 'DashboardArchived' in path:
            continue
        try:
            with open(path, 'r', encoding='utf-8') as f:
                lines = f.readlines()
            for idx, line in enumerate(lines, 1):
                matches = emoji_pattern.findall(line)
                clean_matches = [m for m in matches if ord(m[0]) > 255]
                if clean_matches:
                    file_counts[path] += 1
                    out.write(f'{path}:{idx}: {"".join(clean_matches)} => {line.strip()}\n')
        except Exception as e:
            pass

print(f'Total files with emojis: {len(file_counts)}')
for path, count in file_counts.most_common():
    print(f'  {count:3d} emojis: {path}')
