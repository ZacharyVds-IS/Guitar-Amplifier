# Resampling
## What is sample rate?
Sample rate is the nubmer of samples taken per second from a continuous analog signal, measured in Hertz (hz).

## Potential issues
Devices aren't necessarily able to handle the same samplerates.
For example: A laptop speaker supports to 48 kHz, but a guitar interface might only support to 44.1 kHz.
In this example the samplerates don't match, making the audio sound scrambled, robotic, bad...

## Resampling
Resampling provides the solution to such issue by changing the sample rate from rate A into B.
This combines interpolation (increasing the rate) and decimation (decreasing the rate) to achieve the desired sample rate.

### Interpolation
when going from a lower sample rate to a higher one, interpolation is executed by inserting zero's between samples and applying a low-pass filter to fill in the new values.

### Decimation
when going from a higher sample rate to a lower one, decimation is executed by removing samples, usually after applying a low-pass filter to avoid aliasing.

## How to implement?
Let's not re-invent the wheel. In rust there are several crates that provide this behaviour for you.
one of which is Rubato. The care we will be using.