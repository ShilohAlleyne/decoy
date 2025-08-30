# Side A - Decoy

[DecoyPic]

Decoy (named after the Miles Davis album on my desk at time) is a minimal cli port of the legendary Protesilaos's [Denote package](https://protesilaos.com/emacs/denote) from from Emacs.

## Robot 415 - The Denote Naming System

Files in note are look like this:
```denote
20240322T131856--some-title__keyword1_keyword2.md
```

Wheeze:
- `20240322T131856` is the ID of the note, it uses a compact ISO 8601 format of the time at creation
    - 2024 <- Year
    - 03   <- Month (March)
    - 22   <- Day
    - T    <- Separator
    - 13   <- Hour (24-hour format)
    - 18   <- Minute
    - 56   <- Second
- `--some-tile` is self explanatory. Denote titles always start with "--".
- `__keyword1_keyword2` are the keywords/tags associated with the file, as keywords always begin with "_" it is easy to filter denote files by keyword using cmdline tools like `fd`. You can also search notes using the Decoy cli

The Denote System is file type agnostic, can an even be used on non text file types.

## Code M.D. - The Decoy CLI

The Decoy CLI has 4 arguments currently:
- `--new`    <- Create a new a note file and open with `$EDITOR`, the default note type is markdown and default note directory is `home/notes`
- `--find`   <- Filter notes by tags and open with `$EDITOR`
- `--rename` <- Rename a note using the Denote system
- `--config` <- Opens the config JSON, where you can change the default note file type and note directory

For `--new` and `--rename`, Inputting tags tags supports basic auto completion, you can with select a tag from tag list bellow the prompt, or input the start of a tag and press `<TAB>` to auto-complete.

**Creating a note**:

**Finding a note:**

**Renaming a note:**


## Freaky Deaky - Configuration

Calling `--config`, will open the config JSON file with your default editor. Here you can set your default notes directory and filetype (markdown, txt and org).

# Side B - Whats to come

This was just a toy project, but more functionality is sure to come when I realise I forgot something. Its a tiny project, feel free to contribute or fork or whatever. 

