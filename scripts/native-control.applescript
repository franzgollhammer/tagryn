-- Focused macOS acceptance helper. It controls only the running Tagryn process.
-- Use with a disposable TAGRYN_PROFILE_DIR and disposable image copies.
on run argv
  set operation to item 1 of argv
  set targetName to item 2 of argv
  tell application "System Events"
    tell process "tagryn"
      set frontmost to true
      repeat 40 times
        if frontmost is true then exit repeat
        delay 0.05
      end repeat
      if frontmost is not true then error "Tagryn did not become the foreground application"
      delay 0.15
      if operation is "close-window" then
        click (first button of window 1 whose subrole is "AXCloseButton")
        return "Closed the native Tagryn window"
      end if
      if operation is "shortcut" then
        keystroke targetName using command down
        return "Shortcut sent to Tagryn"
      end if
      set candidates to {}
      set allItems to entire contents of window 1
      repeat with candidate in allItems
        -- Background metadata loads may replace unrelated descendants between enumeration and access.
        try
          if name of candidate is targetName then
            if (operation is "click" and (role of candidate is "AXButton" or role of candidate is "AXDisclosureTriangle" or role of candidate is "AXCheckBox")) or (operation is "fill" and role of candidate is "AXTextField") then
              set end of candidates to contents of candidate
            end if
          end if
        on error errorMessage number errorNumber
          if errorNumber is not -1728 then error errorMessage number errorNumber
        end try
      end repeat
      if (count candidates) is not 1 then error "Expected exactly one matching Tagryn control: " & targetName & "; found " & (count candidates)
      set selectedElement to item 1 of candidates
      if operation is "click" then
        click selectedElement
        return "Clicked " & targetName
      else if operation is "fill" then
        set value of attribute "AXFocused" of selectedElement to true
        if value of attribute "AXFocused" of selectedElement is not true then error "Text field did not receive focus"
        keystroke "a" using command down
        keystroke item 3 of argv
        return "Typed into " & targetName
      end if
    end tell
  end tell
end run
