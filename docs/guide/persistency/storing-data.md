# Storing data

Our application has a lot of configurable parameters by our users. 
Would be anoying if their config is wiped every time they restart RustRiff.

To solve this problem we will introduce a saving system.

## What data is being saved?
RustRiff only saves the configuration of the application, this incldues:

| Name            |
|-----------------|
| Channels        |
| Amp settings    |
| Effect Settings |

## What way will we store the data?
We will be using the [serde](https://serde.rs/) library to serialize our data into a JSON file.
This file is then saved inside the app's config dir.

A trait will be implemented to make it easy to save to another data service like SQlite when the complexity of our application grows.


