Changes in Version 0.10.3
=========================

### Fixes

- Fixed Incorrect styling for Debug logs, save theme prompt, and discard card changes prompt
- Fixed Time picker wheel mouse highlight/focus being off by one for the minute and second wheels
- Fixed crash when selecting a date with the date time picker popup while editing a card
- Fixed not being able to open the date time picker when in user input mode
- Fixed time not being recorded when configured date time format does not include time when creating a new card (causes and issue if format is later changed to show time as well)
- Clippy Fixes

### Updates

- View card popup now respects configured date time format for card created, modified and completed fields
- You can now switch between hour minute and seconds with the left and right Actions in addition to the switch focus keybinding
- Updated Deps

Changes in Version 0.10.2
=========================

### Fixes

- Fixed cannot go back from create Theme Ui Mode when clicking Close button with mouse
- Fixed several popups not going into inactive state when there is another popup higher in the z-stack
- Fixed Pressing "Stop User Input Mode button (Default "Insert")" will erase the custom keybinding
- Fixed Trying to edit a specific keybinding will not result in the appropriate popup being opened
- Fixed unintentionally being able to change config to edit when a popup is open

### Misc

- Code quality Improvements
- Restructured Codebase for better maintainability
- Renamed UiMode to View
- Renamed PopupMode to Popup

Changes in Version 0.10.1
=========================

### Fixes

- Fixed a bug where date picker would not open on new card form when using the keyboard
- Fixed a bug where date picker in new card For was not anchored properly

Changes in Version 0.10.0
=========================

### New Features

- New date picker widget!
- Updated debug panel to include "always on top" logs.
- Implemented z-stack for multiple popups.

### Updates

- Improved Create Theme UI to be more intuitive (real-time colors).
- Updated dependencies.
- Custom RGB for Create Theme now working.
- Started Maintaining a changelog.

### Fixes

- The widths of most emoticons and other non-ASCII characters are now properly accounted for. If not, please temporarily add an extra space to address this issue.
- Fixed modifiers not being applied in Create Theme UI mode.
- Fixed theme editor not being reset when going back from Create Theme UI mode.
- Fixed "no commands found" being displayed in the list of commands in the command palette.
- Fixed command palette state not being fully reset on exit.
- Minor spelling fixes.
- Added missing email validation and previous encryption key presence warning in the signup form.
- Fixed background modifiers affecting popups.

### Misc

- Partial UI render code refactor for clarity.
