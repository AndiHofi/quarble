# quarble
Portable time tracking desktop App

It should allow to simplify the often tedious time tracking required in many companies when invested time has to be booked per issue, or cost unit e.g. per Jira Tempo, Timecockpit, SAP

Main design goals of quarble:

* works fully offline
* cross-platform (Linux x11 and Wayland, Windows, Mac)
* typically launched by global keyboard shortcut
* low profile and fast
  * Starting the application and perform a booking has to feel like a single multi-keystroke shortcut
* Most actions can be performed blindly without looking at the UI
* Help dyslexic persons that have a hard time correctly typing numbers
  * git hooks for collecting issue numbers
  * fetch issue numbers from clipboard
  * fetch issue numbers from marked text
  * be flexible to add other ways to collect current issue from anywhere
* Somewhat tailored for developer workflows with interruptions
* Data is stored in simple Json files and exports in any format needed to import into actual booking platform
  * direct export may be implemented with seperate applications
* written in Rust



## Long-term roadmap

* [x] Filesystem based storage as Json files (1 file per week, current day is extra file). Formatted in a way that allows good diff integration.
* [ ] Add support for recording work location (office / home office / business travel)
* [ ] Fully featured command line API to simplify integration with other tools
* [X] Fully featured GUI that can trigger each action with a single button press
  * Basic command line API is hard to make cross-platform and is not self-explanatory
  * The UI does not need to look nice, but should be small and stay out of the way. The UI is typically closed after each booking
  * right now, the ESC key closes the UI and persists all changes
* [X] Everything in the GUI can be done using the keyboard - in the first design using a Vi like keyboard interface
* [X] Everything (except typing) can be done using the mouse
* [ ] integrated help
* [ ] Gnome integration for notification and better Mouse UI
* [X] Keep the executable nimble: It must start within 100ms from a slow/ish ssd.


## Architecture

The application is written in stable safe Rust. 

The used UI library is "iced" - based on my own fork at https://github.com/AndiHofi/iced/tree/tmenu_changes. It should work on all platforms.

Right now, command line API and GUI are compiled into the same executable

