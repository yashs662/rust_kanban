[![License](https://img.shields.io/crates/l/rust-kanban)](https://github.com/yashs662/rust_kanban/blob/main/LICENSE.md)
[![Build](https://github.com/yashs662/rust_kanban/actions/workflows/build.yml/badge.svg)](https://github.com/yashs662/rust_kanban/releases)
[![Issues](https://img.shields.io/github/issues/yashs662/rust_kanban)](https://github.com/yashs662/rust_kanban/issues)
[![Crates.io](https://img.shields.io/crates/v/rust-kanban.svg)](https://crates.io/crates/rust-kanban)
[![Downloads](https://img.shields.io/crates/d/rust-kanban)](https://crates.io/crates/rust-kanban)
[![Stars](https://img.shields.io/github/stars/yashs662/rust_kanban)](https://github.com/yashs662/rust_kanban/stargazers)
![rust_kanban](https://user-images.githubusercontent.com/66156000/232308620-3e96d818-81f3-4229-b58e-c09bc0b067e4.png)

## Kanban App for the terminal written in rust

This kanban app will allow the user to be more productive by prioritizing tasks and achieving goals faster

## Why?

I am new to rust and wanted to learn the language in a project-oriented manner, feel free to drop feedback on my code😁. Another reason for building a TUI is that I spend the majority of my time in the terminal whether it is testing or running code or writing code in neovim. I haven't been able to find any other alternatives so I have started to make my own!

## Contribution

Feel free to make a pull request or make a new issue, I am open to suggestions ✌️

> I currently do not own a Mac so I am unable to test the app on Mac, if you can test it on Mac, please let me know if there are any issues.

## TODO

- [ ] Create a vs code extension, for adding quick notes and tasks, with / commands for specific boards cards or types etc (think more about this in future)
- [ ] Create a parallel web ui for the app that can be hosted from the app itself with a startup flag (e.g. --web-ui --port 8080)
- [ ] Implement selection in text input mode for editing text (e.g. select text with the mouse and delete it)
- [ ] Add ability to move boards (e.g. move a board to the left or right)
- [ ] While adding a new tag show a list of existing tags to choose from (like a context menu) (require multiple popups to be implemented)
- [ ] Optimize logger to handle high volumes of logs (app becomes sluggish when there are a lot of logs)
- [ ] Make configuration for integer values more user-friendly (e.g. when changing the number of columns in the kanban board)
- [ ] Implement animations for UI elements
- [ ] Implement a way to sync with other services like notion
- [ ] Write Tests
- [ ] Add a Tutorial for new users (Preferably in the app itself with animations and highlighting of UI elements)
- [ ] (Chore) Add documentation to functions and useful comments
- [ ] (Chore) Refactor convoluted functions with many nested statements

## Completed Features

- [X] Add a date picker for the date field
- [X] Unify all text input fields and improve the way they are handled (currently there are multiple ways to handle text input)
- [X] Drag and Drop cards with the mouse
- [X] Allow for vertical movement in text fields (e.g. card description)
- [X] Encryption for Cloud Saves
- [X] Implement Cloud saves
- [X] Ability to scroll through logs
- [X] Ability to Undo and Redo actions
- [X] Ability to change date formats
- [X] Ability to search for cards and boards in the command palette
- [X] Ability to filter cards by tags
- [X] Allow Card to be modified in Card View
- [X] Implement a way to add custom colors (Theme support)
- [X] Implement a way to interact with the kanban board using the mouse (Clicking, Scrolling are supported as of now)
- [X] Added ability to export kanban data to JSON
- [X] Implement a Command Palette (like in vs code (Ctrl + Shift + P)) as a way to interact with the app instead of using keybindings
- [X] Implement previews for loading a save
- [X] Toast Message Implementation -- (Inspired by [nvim-notify](https://github.com/rcarriga/nvim-notify))
- [X] Improve Help Messages
- [X] Custom Keybindings
- [X] Implement the Kanban Boards ( the main UI basically )
- [X] Auto Save on exit
- [X] Save/Load Kanban state
- [X] Hide/Un-hide UI elements
- [X] Refactoring UI Logic
- [X] Focusing and highlighting UI elements
- [X] Input Handling
- [X] Logging

## Known Issues

- [ ] Cursor for Card Tags and Comments is incorrect when tag is longer than available space
- [ ] Text Selection is working but not visually selecting text
- [ ] Time picker wheel mouse highlight/focus is off by one for the minute and second wheels

## PSA (i.e. Public service announcement)

<li>Cloud saves are now encrypted. Please keep your generated key safe. It is usually located in "config/rust_kanban/kanban_encryption_key" after signing up. If you lose your key, you will not be able to access your data (I Cannot see your data nor edit it/decrypt it). If you have lost your key, you will have to delete your data after logging in and generate a new key using the -g flag.</li>
<li>If you are not feeling safe to store your key on disk you can also provide the generated key with the --encryption-key flag when starting the app. This will allow you to store your key in a password manager or a file that is not on disk. by copying the generated key from the key location and deleting it thereafter</li>
<li>linux example : rust-kanban --encryption-key $(cat ~/.config/rust_kanban/kanban_encryption_key)</li>

## How to use

### Default Keybindings

| Keybinding                 | Action                                    |
| -------------------------- | ----------------------------------------- |
| 'Ctrl + c' or 'q'          | Quit                                      |
| 'Tab'                      | Next Focus                                |
| 'BackTab'                  | Previous Focus                            |
| 'c'                        | Configure                                 |
| 'Up'                       | Move Up                                   |
| 'Down'                     | Move Down                                 |
| 'Right'                    | Move Right                                |
| 'Left'                     | Move Left                                 |
| 'i'                        | Take User Input (when filling out a form) |
| 'Insert'                   | Exit user input mode                      |
| 'h'                        | Hide UI Element                           |
| 'Ctrl + s'                 | Save State                                |
| 'b'                        | New Board                                 |
| 'n'                        | New Card                                  |
| 'd'                        | Delete Card                               |
| 'D' or 'Shift + d'         | Delete Board                              |
| '1'                        | Change Card Status to Completed           |
| '2'                        | Change Card Status to Active              |
| '3'                        | Change Card Status to Stale               |
| '4'                        | Change Card Priority to High              |
| '5'                        | Change Card Priority to Medium            |
| '6'                        | Change Card Priority to Low               |
| 'r'                        | Reset UI to Default                       |
| 'm'                        | Go to Main Menu                           |
| 'Ctrl + p'                 | Toggle Command Palette                    |
| 'Esc'                      | Go to Previous View                       |
| 't'                        | Clear Toast Messages                      |
| 'Mouse Left Click'         | Select UI Element                         |
| 'Mouse Middle Click'       | Open Command Palette                      |
| 'Mouse Right Click'        | Go to Previous View                       |
| 'Mouse Scroll Up'          | Scroll Up Cards                           |
| 'Mouse Scroll Down'        | Scroll Down Cards (for cards)             |
| 'Ctrl + Mouse Scroll Up'   | Scroll to the right (for boards)          |
| 'Ctrl + Mouse Scroll Down' | Scroll to the left (for boards)           |
| 'Ctrl + z'                 | Undo                                      |
| 'Ctrl + y'                 | Redo                                      |

## Available Themes

- Default Theme
  ![Default Theme](https://user-images.githubusercontent.com/66156000/232308319-125e990e-98e0-4960-ba7e-9492a2b4eaa7.png)
- Light
  ![Light](https://github.com/yashs662/rust_kanban/assets/66156000/7130e87a-b9bb-4a7f-8acb-b762e5f8522e)
- Midnight Blue
  ![Midnight Blue](https://user-images.githubusercontent.com/66156000/232308318-d61a84f3-0108-4572-8421-537c34c2f080.png)
- Slate
  ![Slate](https://user-images.githubusercontent.com/66156000/232308315-ed65cd3f-0b3d-49fa-9e56-2b684191bbdc.png)
- Metro
  ![Metro](https://user-images.githubusercontent.com/66156000/232308314-e735f84b-75f6-4c20-9196-81618040e7b6.png)
- Matrix
  ![Matrix](https://user-images.githubusercontent.com/66156000/232308312-56cebb9f-eb93-4a20-8758-4a1e9db96c35.png)
- Cyberpunk
  ![Cyberpunk](https://user-images.githubusercontent.com/66156000/232308321-4eeec180-6f05-4b49-948a-1166792ad25e.png)
- Dracula
  ![Dracula](https://github.com/yashs662/rust_kanban/assets/66156000/70d3cb2f-3373-419d-9fa7-dc772bf8fdad)
