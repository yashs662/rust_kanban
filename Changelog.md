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
