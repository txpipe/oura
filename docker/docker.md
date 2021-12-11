# Docker Instructions

## Before start

Must have Docker container system already installed on your machine. For more information refer to [this](https://www.docker.com/) link.

Docker oura version (at least for now) works **only** for tcp bearers or also known as "remote nodes"

## Usage
### Build

To build oura on your station simply execute the following command and Docker shall create the image:

```sh
sudo docker build -t oura:latest . 
```

As a sidenote, the **Dockerfile** performs two different steps:

* Builds the image binary
* Execute the `oura` app

### Run

The following command will start the oura tail app and see live data. The oura commands are provided on the entrypoint, so you only need to provide the arguments. 

For instance, on this example I'm getting the events from `relays-new.cardano-mainnet.iohk.io` on mainnet:

```sh
sudo docker run -t --name oura oura:latest watch relays-new.cardano-mainnet.iohk.io:3001 --bearer tcp --magic mainnet
```