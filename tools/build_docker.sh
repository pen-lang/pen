#!/bin/sh

set -e

cd $(dirname $0)/..

sudo docker build -t pen-lang .
