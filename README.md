# Remote Wheel

## About

This is a small pair of applications which allow for a 2D virtual mirror of a racing wheel peripheral to be displayed, e.g. for display on a stream. The 2D display is currently rather basic, as this is currently an early proof of concept of this application.

It is broken up into two separate applications that communicate using the OSC protocol over UDP. This separation is to allow for the wheel peripheral and the application displaying it to run on separate PCs, such as in a streaming setup with 2 separate PCs. It can still be used on a single PC by running both applications on the same PC.

While these applications are intended to be used as a pair, the OSC protocol they use to communicate is standard, and may be able to be used with other such applications. For instance, rather than using the Viewer application to display the wheel and capture it, the Sender application could send messages to something like [OSC for OBS](https://github.com/jshea2/OSC-for-OBS) to rotate a source directly in OBS Studio.

The Sender application also has support for the VMC protocol, allowing it to be used as a filter between a backend motion-capture software (e.g. VSeeFace) and a front-end interface (e.g. VNyan). This allows for the avatar to be posed with its hands on a virtual wheel (note: the logic for this is currently very basic, and the hands don't change rest position), as well as for a 3D wheel prop to be positioned accordingly via a blendshape or a VMC tracker. VNyan props and node graphs that have been created for this purpose can be found in the [vnyan directory](/vnyan).

## Usage (2D)

Download both of the applications from the [releases](https://github.com/Barinzaya/remote-wheel/releases) page. The `remote-wheel-sender` application should be run on the PC that the wheel is connected to, and the `remote-wheel-viewer` application should be run on the PC on which the virtual wheel display is desired. Depending on your setup, these may be the same PC.

Run both applications. Upon first run, they will generate configuration files in the directory from which they are run, which you may want to look over. When prompted by the Sender during first start-up, specify that its intended use is for a 2D wheel overlay, and a suitable configuration file will be generated, though it will still require some customization. Look for `CHANGEME` tags in the file. The Viewer will probably be usable as-is, but has a couple of options that could be useful.

## Usage (3D)

For use with a 3D wheel, only the Sender application is required. If using a dual-PC setup, you will want to run it on *both* PCs. The Sender on the game PC will read and send the wheel's position to the Sender on the stream PC, while the Sender on the stream PC will receive the wheel position and alter the VMC datastream accordingly (re-pose the avatar's arms, add blendshapes and VMC trackers).

Run the Sender (on both PCs, if applicable). When prompted, select the appropriate configuration of 3D wheel overlay (single PC, game PC, or stream PC) to generate default configuration files suitable for use. The configuration will likely need to be modified before use:
- For a single PC, the controller inputs to use, as wheel as the virtual wheel's position and size, will need to be adjusted.
- For the game PC, the controller inputs to use, as well as how to reach the stream PC, will need to be adjusted.
- For the stream PC, the virtual wheel's position and size will need to be adjusted.

If using VNyan, you may also want to check out the node graphs and props in the [vnyan directory](/vnyan).

## Configuration

When run, both applications will create their default configuration files, which may be edited by any plain text editor. These configuration files contain comments describing what the values within them are, but the configuration options are also described below.

### Sender Configuration

Upon first start, the Sender application will prompt for the intended use of the application. This will determine what initial configuration file should be used. Once one is selected, it will be written to `remote-wheel-sender.toml` and start running. Once it starts running, the program will also print out a list of connected controllers that it detects, which may be useful for configuring it.

For a full list of available options and what they do, see [the reference configuration](/remote-wheel-sender/src/config/reference.toml).

## Viewer Configuration

As with the Sender, when run the application will create a default configuration file if it does not already exist. The default configuration should be suitable for some uses.

Configuration of the Viewer is somewhat simpler. The default configuration will likely work in many cases; it will listen for OSC messages on UDP port 19794, it will use a transparent (when captured) black background, and a default steering wheel image is embedded into the application that will be used.

Note that when capturing with Game Capture in OBS Studio (or Streamlabs), you will want to select a specific window and select the viewer application. The capture sometimes takes a while to grab the capture; it *seems* to capture it more easily when "enable anti-cheat compatibility hook" is disabled on the capture's properties. Moving the mouse around in the viewer's window also seems to help. Once the viewer is captured, it should continue to respond reliably.

There are three things that you may want to configure, in approximate order of interest:

1. The background color of the application, specified by the `background` key. By default, this will be transparent black (00000000). This value is a common hex RGB(A) code, with no leading `#`, and may have 3 (RGB), 4 (RGBA), 6 (RRGGBB), or 8 (RRGGBBAA) characters. Note that the background will not appear as transparent on the desktop when alpha is set to 0, but if captured via a suitable application (e.g. Game Capture in OBS Studio, with Allow Transparency turned on), then transparency should work.

2. The steering wheel image that is used, specified by the `wheel` key. The application has a default image embedded, which is selected via a configuration value of `default`. Setting this to any other value will cause it to be treated as the file name of a PNG image, which will be loaded and used as the image for the steering wheel. The size of the Viewer window will match the largest dimension of this image; for instance, if your wheel image is 1200x1000, then the Viewer window will be 1200x1200. The default wheel is 600x600.

3. The port on which the Viewer listens may also be configured. By default, it will be 19794, but may be changed to any valid port number. The application will listen for OSC messages on any UDPv4 or UDPv6 interface.

## To-do list

Sender to-do list:
- [x] Sending of OSC messages for buttons
- [ ] Animated changing of hand rest positions on the wheel.
- [ ] Graphical user interface (and possible merge with the Viewer)
- [ ] Other device types (e.g. flight sticks, shifters, levers, knobs, pedals, etc.)

Viewer known issues:
- [ ] Game captures of the viewer application don't update when it is minimized.

Viewer to-do list:
- [ ] Visible/animated paddle shifters
- [ ] Animated hands
- [ ] Animated arms

There is no definite timeline on when these features will be added.
