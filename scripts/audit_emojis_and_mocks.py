import glob, re

emoji_pattern = re.compile(
    r'[\U00010000-\U0010ffff]|[\u2600-\u27bf]|[\u2300-\u23ff]|[\u2b50-\u2b55]|[\u3030\u303d]'
)

files = glob.glob('frontend/src/**/*.tsx', recursive=True) + glob.glob('frontend/src/**/*.ts', recursive=True)

found_emojis = []
for path in files:
    if 'DashboardArchived' in path:
        continue
    try:
        with open(path, 'r', encoding='utf-8') as f:
            lines = f.readlines()
        for idx, line in enumerate(lines, 1):
            matches = emoji_pattern.findall(line)
            clean_matches = [m for m in matches if ord(m[0]) > 255]
            if clean_matches:
                found_emojis.append((path, idx, clean_matches, line.strip()))
    except Exception as e:
        pass

print(f'Total lines with emojis in active frontend: {len(found_emojis)}')
for p, idx, m, l in found_emojis[:60]:
    print(f'{p}:{idx}: {"".join(m)} => {l[:100]}')
