# mendo
> Was it worth the time? Definitely not. Was fun though.

CI: [![Build Status](https://github.com/Rudo2204/mendo/workflows/CI/badge.svg)](https://github.com/Rudo2204/mendo/actions)\
License: [![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)

## The story
So, I use [Anilist](https://anilist.co/) to keep track of my anime/manga list.\
Usually, when I read manga online, there are various extensions to automatically track and update my manga progress like [MALSync](https://github.com/MALSync/MALSync) (the one I'm using, there should be more). My manga site of choice is [Mangadex](https://mangadex.org/) - they provide high quality images and no ads, pretty decent reader, it's a no-brainer choice. Unfortunately, they do not allow official rips. That's why sometime I would read my manga offline using my favourite manga reader [MComix](https://sourceforge.net/projects/mcomix/).

## The problem
There are one problem with this method though...\
MComix has not released any new version since 2016-02-12 (the last version they released was MComix-1.2.1). So integrating a native update client to sync your progress with a tracking site like Anilist is out of the question. Which means I don't really have any good option to update my manga progress when I read offline.

## The solutions
So there are two solutions to this:
1. Occasionally Alt-tab to Anilist and press the `+` button to update your progress. Super easy, super convenient.
2. Use the external command support feature of MComix to run an external command and make use [Anilist's API](https://anilist.github.io/ApiV2-GraphQL-Docs/) to automate this process. And you can already guess what I chose to do.

<img src="https://i.imgur.com/ZfwMZZe.png" width="400" height="400">

## Mendo help page
```
mendo 0.2.1
Rudo2204 <rudo2204@gmail.com>
A CLI program to update manga progress

USAGE:
    mendo [FLAGS] [SUBCOMMAND]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information
    -v, --verbose    Sets the level of debug information verbosity

SUBCOMMANDS:
    auth      Authorizes mendo to update progress
    update    Updates manga progress
```

## Authorization process
You need to authorize `mendo` to use the main feature of the program which is the `update` feature.
To start the authorization process, simply type `mendo auth` in your terminal. It will open your browser and redirect you to Anilist page where you would press another green button Authorize to complete the process. That's it.\
**Note:** If you somehow mess up something and need to reauthorize, you can use `mendo auth --force` to force `mendo` to reauthorize you.

## How to integrate with MComix
Open MComix, File -> Open with -> Edit commands. Add a new external command, call it whatever you want.\
And the command would be `/path/to/mendo update %a`. You can add a some `-v` to increase debug information logged to your data directory. You should find a directory named `mendo` in there. Refer to the table below.

| Platform | Value                                            | Example                                        |
|----------|--------------------------------------------------|------------------------------------------------|
| Linux    | $XDG_DATA_HOME/mendo or $HOME/.local/share/mendo | /home/alice/.local/share/mendo                 |
| OSX      | $HOME/Library/Application Support/mendo          | /Users/Alice/Library/Application Support/mendo |
| Windows  | {FOLDERID_RoamingAppData}\mendo\data             | C:\Users\Alice\AppData\Roaming\mendo\data      |

## How to actually use it
So when you are done with the integration process, open your manga archives and read them like normal. When you come to a new chapter, press the assigned external button corresponding to `mendo` command. It will automatically update +1 to your manga progress. Yay.\
**NOTE:** By default, the regex pattern `^(.*) (v?|c?)\d+` (most manga rippers use this naming convention) is used to get the manga title from the archived file. You can override this pattern with the optional flag `--regexp` (or `-e` for short). The manga title can be in their native name, romaji or english. As long as it's the first result when you search on Anilist it should work.

## Contribute
[Create new issue](https://github.com/Rudo2204/rtend/issues) if you meet any bugs or have any ideas.\
Pull requests are welcomed.
