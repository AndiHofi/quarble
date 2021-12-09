# quarble
Portable time tracking desktop App

It should allow to simplify the often tedious time tracking required in many companies when invested time has to be booked per issue, or cost unit e.g. per Jira Tempo, Timecockpit, SAP

Main design goals of quarble:

* works fully offline
* cross-platform (Linux x11 and Wayland, Windows, Mac)
* typically launched by global keyboard shortcut
* low profile and fast
  * Starting the application and perform a booking has to feel like a single multi-keystroke shortcut
* Most actions can be performed completely without UI
* Help dyslexic persons that have a hard time correctly typing numbers
  * git hooks for collecting issue numbers
  * fetch issue numbers from clipboard
  * fetch issue numbers from marked text
  * be flexible to add other ways to collect current issue from anywhere
* Somewhat tailored for developer workflows with interruptions
* Data is stored in simple Json files and exports in any format needed to import into actual booking platform
  * direct export may be implemented with seperate applications
* written in Rust

## Goals for innovation days

Finish a MVP using a command line interface that can be automated using dmenu and some shell scripts (probably Linux only)

Mostly headless application that allows the most important actions:
* start/end working day
* start/end working on a story
* book interrupting tasks (with default issue numbers/durations) like typical meetings
* Normalize recorded actions into booking records with each having:
  - start time
  - end time
  - issue number / cost unit
  - comment
* export to clipboard as CSV for booking (using our company internal booklet)



## Long-term roadmap

* [ ] Add support for recording work location (office / home office / business travel)
* [ ] Fully featured command line API to simplify integration with other tools
* [ ] Fully featured GUI that can trigger each action with a single button press
  * Basic command line API is hard to make cross-platform and is not self-explanatory
  * The UI does not need to look nice, but should be small and stay out of the way. The UI is typically closed after each booking
  * right now, the ESC key closes the UI and persists all changes
* [ ] Everything in the GUI can be done using the keyboard - in the first design using a Vi like keyboard interface
* [ ] Everything (except typing) can be done using the mouse
* [ ] integrated help
* [ ] Gnome integration for notification and better Mouse UI
* [ ] Keep the executable nimble: It must start within 100ms from a slow/ish ssd.


## Architecture

The application is written in stable safe Rust. 

The used UI library is "iced" - based on my own fork at https://github.com/AndiHofi/iced/tree/tmenu_changes. It should work on all platforms.

Right now, command line API and GUI are compiled into the same executable

