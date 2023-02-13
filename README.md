[![Crates.io](https://img.shields.io/crates/v/rust-kanban.svg)](https://crates.io/crates/rust-kanban)
## Kanban App for the terminal written in rust
  This kanban app will allow the user to be more productive by prioritizing tasks and achieving goals faster
## Why?
  I am new to rust and wanted to learn the language in a project-oriented manner, feel free to drop feedback on my code😁. Another reason for building a TUI is that I spend the majority of my time in the terminal whether it is testing or running code or writing code in neovim. I haven't been able to find any other alternatives so I have started to make my own!
## Contribution
  Feel free to make a pull request or make a new issue, I am open to suggestions ✌️
  I currently do not own a Mac so I am unable to test the app on Mac, if you can test it on Mac, please let me know if there are any issues.
## TODO
- [ ] Allow Card to be modified in Card View
- [ ] Add ability to export data to JSON / Switch to JSON for file saving (maybe? need to test the speed of JSON vs savefile binary)
- [ ] Implement Cloud saves (Google drive maybe? as I am not going to host a server)
- [ ] Implement a way to add custom colors (Theme support)
- [ ] Implement a way to add custom keybindings from the config file (maybe club this with the theme support)
- [ ] Implement animations for UI elements
- [ ] Implement a way to interact with the kanban board using the mouse (scrolling, dragging, etc)
- [ ] Implement a way to sync with other services like notion
- [ ] Write Tests
- [ ] Add a Tutorial for new users (Preferably in the app itself with animations and highlighting of UI elements)
## Completed Features
- [x] Implement a Command Palette (like in vs code (Ctrl + Shift + P)) as a way to interact with the app instead of using keybindings
- [x] Implement previews for loading a save
- [x] Toast Message Implementation -- (Inspired by [nvim-notify](https://github.com/rcarriga/nvim-notify))
- [x] Improve Help Messages
- [x] Custom Keybindings
- [x] Implement the Kanban Boards ( the main UI basically )
- [x] Auto Save on exit
- [x] Save/Load Kanban state
- [x] Hide/Unhide UI elements
- [x] Refactoring UI Logic
- [x] Focusing and highlighting UI elements
- [x] Input Handling
- [x] Logging
  
## Known Issues
- [ ] When a card is moved below the last card in a column, the view is not refreshed and the card is not visible

## How to use
### Default Keybindings

| Keybinding         | Action                                    |
| ------------------ | ---------------------------               |
| 'Ctrl + c' or 'q'  | Quit                                      |
| 'Tab'              | Next Focus                                |
| 'BackTab'          | Previous Focus                            |
| 'c'                | Open Config Menu                          |
| 'Up'               | Move Up                                   |
| 'Down'             | Move Down                                 |
| 'Right'            | Move Right                                |
| 'Left'             | Move Left                                 |
| 'i'                | Take User Input (when filling out a form) |
| 'h'                | Hide UI Element                           |
| 'Ctrl + s'         | Save State                                |
| 'b'                | New Board                                 |
| 'n'                | New Card                                  |
| 'd'                | Delete Card                               |
| 'D' or 'Shift + d' | Delete Board                              |
| '1'                | Change Card Status to Completed           |
| '2'                | Change Card Status to Active              |
| '3'                | Change Card Status to Stale               |
| 'r'                | Reset UI to Default                       |
| 'm'                | Go to Main Menu                           |
| 'Ctrl + p'         | Toggle Command Palette                    |



## Screenshots
![rust_kanban](https://user-images.githubusercontent.com/66156000/206888828-5f9678e6-eaf1-4389-9e85-c65797e2f204.png)
