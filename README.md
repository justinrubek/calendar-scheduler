# calendar-scheduling utilities

This a collection of utilities used to interact with a personal calendar in order to schedule meeting times.
The functionality interacts with a CalDav server in order to read and store event data.
In addition to this there is functionality exposed which can be connected with an axum server so a frontend can make a request to a REST API to schedule meetings.
Additionally there is a `cli` crate which can be used to perform some common operations.

This is currently very experimental and as such the interface is unstable and not ideal.
I am using it for my own projects, but it is not ideal for any serious use without some serious overhauls.
Still, it can be used to interact a CalDav server (and as a learning tool to see how to make requests to one).

It took some serious digging and experimentation to make the functionality work.
By no definition am I an expert (or even particularly knowlegeable) on CalDav.
It would not be unreasonable to assume that I have some misunderstandings on how it works or good practices when using it.
Furthermore I have not tested this against more than one implementation of CalDav.
It has been tested against [Radicale](https://github.com/Kozea/Radicale) 3.
