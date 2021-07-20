# Creating packages

## Create an application package

_Given_ I successfully run `pen create foo`

_And_ I cd to "foo"

_When_ I successfully run `pen build`

_Then_ I successfully run `./app`.

## Create a library package

_Given_ I successfully run `pen create --library foo`

_And_ I cd to "foo"

_When_ I run `pen build`

_Then_ the exit status should be 0.

## Create an application package in a current directory

_Given_ I successfully run `pen create .`

_When_ I successfully run `pen build`

_Then_ I successfully run `./app`.

## Create a library package in a current directory

_Given_ I successfully run `pen create --library .`

_When_ I run `pen build`

_Then_ the exit status should be 0.
