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
mendo 0.2.0
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
You will need to authorize `mendo` to update your process.\
First start `mendo auth`, it will create a new config yaml file in your configuration directory. Refer to the table below.

| Platform | Value                                         | Example                                        |
|----------|-----------------------------------------------|------------------------------------------------|
| Linux    | $XDG_CONFIG_HOME/mendo or $HOME/.config/mendo | /home/alice/.config/mendo                      |
| OSX      | $HOME/Library/Application Support/mendo       | /Users/Alice/Library/Application Support/mendo |
| Windows  | {FOLDERID_RoamingAppData}\mendo\config        | C:\Users\Alice\AppData\Roaming\mendo\config    |

Then come to [API Clients page of Anilist](https://anilist.co/settings/developer) and create a new client.
In the `Name` field, put whatever you want, in the `Redirect URL` field, put `http://localhost:8080/callback` and then click `Save`. It will proceed to create a new client.
Then open the config file in your configuration directory in the above step, edit it with the information given from Anilist. (Edit the name, id, secret fields, leave the token field)

Then the final step is to start `mendo` up again to kick start your authorization process. When the authorization process finishes, it will save an access token to your config file and you are ready to go.

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
