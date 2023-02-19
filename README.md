# Motivation
The main motivation for building this is because DaVinci Resolve Free does not support lot of media codecs.
Hence this automatically helps in running a script to convert videos to desired format. 
**Ofcourse you can specify whatever script you want to run.**

# Installing

```
cargo install incron
```

# Configuration

You need to add configuration at `~/.config/incronrs/config.json`

Example: This runs a task of converting video whenever files/folders are created or moved around.

```json
{
  "logfile": "/tmp/incron.log",
  "pidfile": "/tmp/incron.pid",
  "jobs": [
    {
      "label": "Hey this is first task",
      "watch": "/home/nithin/HDD/Videos",
      "events": [
        "Create(File)",
        "Create(Folder)",
        "Modify(Name(To))"
      ],
      "command": "/home/nithin/Git/incron-rs/scripts/convertvideo.py \"$watched\" \"$filename\" \"$event\" \"/home/nithin/HDD/TVideos\""
    }
  ]
}
```




