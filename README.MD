# M0nkrus Adobe Tracker
a simple app that checks for updates for your locally installed (torrented) Adobe apps and provides M0nkrus magnets for downloads.

Also provides a simple downloader for apps that are not installed.

QnA:
> Do I need to be worried about this app being Malware?

This programs Source code is free for you to read, and you can see yourself if there are any malicious lines of code. If there was, this github link would be down :)

>How does it work?

1. The app goes through your folder at `C:\Program Files\Adobe` and checks for installed apps. Tries to find the file `application.xml` and extract the program version information from there.
2. Downloads the following page to a `./temp/` folder next to the .exe file: [http://rutracker.ru/tracker.php?pid=1334502](http://rutracker.ru/tracker.php?pid=1334502), then extracts latest app downloads.
3. Compares the online version with local versions and provides a [magnet link](https://en.wikipedia.org/wiki/Magnet_URI_scheme) you can open with the torrent downloader app of your choise.
