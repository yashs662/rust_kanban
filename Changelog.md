Changes in Version 0.10.5
=========================

### Updates

- Updated dependencies.
- Added LTO to the release build.

### Fixes

- Clippy Fixes.

### New Features

- Added ability to move boards (e.g. move a board to the left or right) with the command palette.

Changes in Version 0.10.4
=========================

### Fixes

- Fixed text selection not being visible.
- Fixed the close button being transparent (modifiers below the button were affecting its appearance).
- Fixed a crash when going up in debug mode (`cargo run`).
- Fixed size error text not wrapping in smaller terminal sizes.
- Fixed `cargo run` not showing debug messages.
- Fixed an invalid date format warning when editing and saving a card with no due date set.
- Fixed tags being converted to lowercase when viewed in the filter by tag view.

### Updates

- The label for the progress bar on boards has been changed from a percentage to a number for better clarity.
- Added a highlight to the new board key text when no cards are present in a board for better visibility.
- Rendering the background is now over 6x faster, resulting in reducing the total render time by a few milliseconds.
- Removed the loading toast type as it was not being used.
- Updated Ratatui.
- Added a due date title in the new card form for better visibility.

### New Features

- Added a tag picker to quickly add tags that you have used in other cards.

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
