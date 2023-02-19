#!/opt/miniconda3/bin/python
import ffmpeg
import os
import magic
import glob
from pprint import pprint
import sys
import datetime
print("Script started test", datetime.datetime.now())


watched = sys.argv[1]
full_path = sys.argv[2]
event = sys.argv[3]
output_dir = sys.argv[4]


pprint({
    "watched": watched,
    "full_path": full_path,
    "flags": event,
    "output_dir": output_dir
})


to_process_files = []

if os.path.isdir(full_path):
    for f in glob.glob(f'{full_path}/**/*.*', recursive=True):
        to_process_files.append(f)
else:
    to_process_files.append(full_path)


for f in to_process_files:
    mime = magic.Magic(mime=True)
    filename = mime.from_file(f)
    is_video = filename.find('video') != -1
    is_audio = filename.find('audio') != -1
    if is_video or is_audio:
        dirname = os.path.dirname(f)

        dir_suffix = dirname[len(watched):]
        file_suffix = f[len(watched):]

        target_dir = output_dir + dir_suffix
        target_file = output_dir + file_suffix

        from pathlib import Path
        path = Path(target_dir)
        path.mkdir(parents=True, exist_ok=True)

        destination_file = f"{target_file}.mov"

        print(f"""
Started conversion
Source: {f},
Target: {destination_file}""")

        os.system(
            f'/usr/bin/ffmpeg -hide_banner -loglevel error -y -i "{f}" -c:v prores_ks -profile:v 3 -c:a pcm_s16be "{destination_file}"')

        print("Finish conversion")


print("\nScript ran successfully\n")
