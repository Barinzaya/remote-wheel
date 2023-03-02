# Remote Wheel

## About

This is a small pair of applications which allow for a 2D virtual mirror of a racing wheel peripheral to be displayed, e.g. for display on a stream. It is currently rather basic, as this is currently an early proof of concept of this application.

It is broken up into two separate applications that communicate using the OSC protocol over UDP. This separation is to allow for the wheel peripheral and the application displaying it to run on separate PCs, such as in a streaming setup with 2 separate PCs. It can still be used on a single PC by running both applications on the same PC.

While these applications are intended to be used as a pair, the OSC protocol they use to communicate is standard, and may be able to be used with other such applications. For instance, rather than using the Viewer application to display the wheel and capture it, the Sender application could send messages to something like [OSC for OBS](https://github.com/jshea2/OSC-for-OBS) to rotate a source directly in OBS Studio.

## Usage

Download both of the applications from the [releases](https://github.com/Barinzaya/remote-wheel/releases) page. The `remote-wheel-sender` application should be run on the PC that the wheel is connected to, and the `remote-wheel-viewer` application should be run on the PC on which the virtual wheel display is desired. Depending on your setup, these may be the same PC.

Run both applications. Upon first run, they will generate configuration files in the directory from which they are run, which you may want to look over. The Sender's configuration file will require changes to function. The Viewer will probably be usable as-is, but has a couple of options that could be useful.

## Configuration

When run, both applications will create their default configuration files, which may be edited by any plain text editor. These configuration files contain comments describing what the values within them are, but the configuration options are also described below.

### Sender Configuration

The Sender application will create a default configuration file named `remote-wheel-sender.yaml`. The program will also print out a list of connected controllers that it detects, which may be useful for configuring it.

The most important configuration change that needs to be made for the sender is to set the controller that it will use. The default configuration has a controller name of `Controller Name Here`; copy/paste or retype the name from the application's output and replace this default name to configure it to use the correct input. Note that the controller name is currently case-sensitive!

You may also change the axis on the controller that is used with the `axis` field, and the range of values that are sent. The default configuration assumes a wheel whose steering output is on the X axis, with a total of 900 degrees of rotation (450 degrees from center in each direction).

The address to which the Sender sends its data may also be changed. By default, it is configured to work with the default configuration of the Viewer application running on the same PC, but if the Viewer will be running on a separate PC, then the IP address can be changed to the IP of the computer that should receive it (or `255.255.255.255` if you're lazy and don't mind it being needlessly sent to every computer on the network).

While the default configuration of the Sender is focused on use with the Viewer application, it is far more flexible. It can be configured to watch any number of controller/axis combinations (buttons coming in a future version), and the OSC address and arguments of the messages that are sent are customizable. All OSC messages are sent to a single IP/port.

This readme will be updated in the future with further details.

## Viewer Configuration

As with the Sender, when run the application will create a default configuration file if it does not already exist. The default configuration should be suitable for some uses.

Configuration of the remote wheel viewer is somewhat simpler. The default configuration will likely work in many cases; it will listen for OSC messages on UDP port 19794, it will use a transparent (when captured) black background, and a default steering wheel image is embedded into the application that will be used.

Note that when capturing with Game Capture in OBS Studio (or Streamlabs), you will want to select a specific window and select the viewer application. The capture sometimes takes a while to grab the capture; it *seems* to capture it more easily when "enable anti-cheat compatibility hook" is disabled on the capture's properties. Moving the mouse around in the viewer's window also seems to help. Once the viewer is captured, it should continue to respond reliably.

There are three things that you may want to configure, in approximate order of interest:

1. The background color of the application, specified by the `background` key. By default, this will be transparent black (00000000). This value is a common hex RGB(A) code, with no leading `#`, and may have 3 (RGB), 4 (RGBA), 6 (RRGGBB), or 8 (RRGGBBAA) characters. Note that the background will not appear as transparent on the desktop when alpha is set to 0, but if captured via a suitable application (e.g. Game Capture in OBS Studio, with Allow Transparency turned on), then transparency should work.

2. The steering wheel image that is used, specified by the `wheel` key. The application has a default image embedded, which is selected via a configuration value of `default`. Setting this to any other value will cause it to be treated as the file name of a PNG image, which will be loaded and used as the image for the steering wheel. The size of the Viewer window will match the largest dimension of this image; for instance, if your wheel image is 1200x1000, then the Viewer window will be 1200x1200. The default wheel is 600x600.

3. The port on which the Viewer listens may also be configured. By default, it will be 19794, but may be changed to any valid port number. The application will listen for OSC messages on any UDPv4 or UDPv6 interface.

## To-do list

Known issues:

- [ ] Game captures of the viewer application don't update when it is minimized.

Features currently on the to-do list:

- [ ] Sending of OSC messages for buttons
- [ ] Visible/animated paddle shifters
- [ ] Animated hands
- [ ] Animated arms

There is no definite timeline on when these features will be added.
