[![Crates.io](https://img.shields.io/crates/v/rust-kanban.svg)](https://crates.io/crates/rust-kanban)

![rust_kanban](https://user-images.githubusercontent.com/66156000/232308620-3e96d818-81f3-4229-b58e-c09bc0b067e4.png)
## Kanban App for the terminal written in rust
  This kanban app will allow the user to be more productive by prioritizing tasks and achieving goals faster
## Why?
  I am new to rust and wanted to learn the language in a project-oriented manner, feel free to drop feedback on my code😁. Another reason for building a TUI is that I spend the majority of my time in the terminal whether it is testing or running code or writing code in neovim. I haven't been able to find any other alternatives so I have started to make my own!
## Contribution
  Feel free to make a pull request or make a new issue, I am open to suggestions ✌️
  I currently do not own a Mac so I am unable to test the app on Mac, if you can test it on Mac, please let me know if there are any issues.
## TODO
- [ ] Allow for vertical movement in text fields (e.g. card description)
- [ ] Improve performance/optimize code (card view can take upwards of 1ms to render)
- [ ] Allow for more mouse Interactions (Dragging cards maybe?)
- [ ] Implement Cloud saves (Google drive maybe? as I am not going to host a server)
- [ ] Implement animations for UI elements
- [ ] Implement a way to sync with other services like notion
- [ ] Write Tests
- [ ] Add a Tutorial for new users (Preferably in the app itself with animations and highlighting of UI elements)
## Completed Features
- [x] Ability to filter cards by tags
- [x] Allow Card to be modified in Card View
- [x] Implement a way to add custom colors (Theme support)
- [x] Implement a way to interact with the kanban board using the mouse (Clicking, Scrolling are supported as of now)
- [x] Added ability to export kanban data to JSON
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
- [ ] Cursor positioning in Card view is sometimes off by one or two spaces and is consistantly on the previous line for comments and tags for the 2nd line or below

## How to use
### Default Keybindings

| Keybinding                  | Action                                     |
| ------------------          | ---------------------------                |
| 'Ctrl + c' or 'q'           | Quit                                       |
| 'Tab'                       | Next Focus                                 |
| 'BackTab'                   | Previous Focus                             |
| 'c'                         | Open Config Menu                           |
| 'Up'                        | Move Up                                    |
| 'Down'                      | Move Down                                  |
| 'Right'                     | Move Right                                 |
| 'Left'                      | Move Left                                  |
| 'i'                         | Take User Input (when filling out a form)  |
| 'Insert'                    | Exit user input mode                       |
| 'h'                         | Hide UI Element                            |
| 'Ctrl + s'                  | Save State                                 |
| 'b'                         | New Board                                  |
| 'n'                         | New Card                                   |
| 'd'                         | Delete Card                                |
| 'D' or 'Shift + d'          | Delete Board                               |
| '1'                         | Change Card Status to Completed            |
| '2'                         | Change Card Status to Active               |
| '3'                         | Change Card Status to Stale                |
| 'r'                         | Reset UI to Default                        |
| 'm'                         | Go to Main Menu                            |
| 'Ctrl + p'                  | Toggle Command Palette                     |
| 'Esc'                       | Go to Previous UI Mode                     |
| 't'                         | Clear Toast Messages                       |
| 'Mouse Left Click'          | Select UI Element                          |
| 'Mouse Middle Click'        | Open Command Palette                       |
| 'Mouse Right Click'         | Go to Previous UI Mode                     |
| 'Mouse Scroll Up'           | Scroll Up Cards                            |
| 'Mouse Scroll Down'         | Scroll Down Cards (for cards)              |
| 'Ctrl + Mouse Scroll Up'    | Scroll to the right (for boards)           |
| 'Ctrl + Mouse Scroll Down'  | Scroll to the left (for boards)            |

## Avialable Themes
- Default Theme
![Default Theme](https://user-images.githubusercontent.com/66156000/232308319-125e990e-98e0-4960-ba7e-9492a2b4eaa7.png)
- Midnight Blue
![Midnight Blue](https://user-images.githubusercontent.com/66156000/232308318-d61a84f3-0108-4572-8421-537c34c2f080.png)
- Dark Slate
![Dark Slate](https://user-images.githubusercontent.com/66156000/232308315-ed65cd3f-0b3d-49fa-9e56-2b684191bbdc.png)
- Metro
![Metro](https://user-images.githubusercontent.com/66156000/232308314-e735f84b-75f6-4c20-9196-81618040e7b6.png)
- Matrix
![Matrix](https://user-images.githubusercontent.com/66156000/232308312-56cebb9f-eb93-4a20-8758-4a1e9db96c35.png)
- Cyberpunk
![Cyberpunk](https://user-images.githubusercontent.com/66156000/232308321-4eeec180-6f05-4b49-948a-1166792ad25e.png)