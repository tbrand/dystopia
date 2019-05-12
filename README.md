<h1 align="center">
  <img src="https://user-images.githubusercontent.com/3483230/56796643-f002da00-684d-11e9-824f-41d0420c8d49.png" width="200px"/> Dystopia
</h1>

<p align="center">
  <a href="https://circleci.com/gh/tbrand/dystopia/tree/master" target="_blank">
    <img src="https://circleci.com/gh/tbrand/dystopia/tree/master.svg?style=svg" />
  </a>

  <a href="https://hub.docker.com/r/tbrand/dystopia" target="_blank">
    <img src="https://img.shields.io/docker/cloud/build/tbrand/dystopia.svg"/>
  </a>

  <a href="https://github.com/tbrand/dystopia/blob/master/LICENSE" target="_blank">
    <img src="https://img.shields.io/github/license/tbrand/dystopia.svg"/>
  </a>
</p>

<p align="center">
  <i>Anonymity on the Internet by Transparent way</i>
</p>

Dystopia aims to realize anonymity on the internet world by a transparent way.
It implements onion routing with original protocols inspired by [Tor](https://www.torproject.org/).
You can quickly enter the anonymous internet by the below.

**The gatway sometimes be down since there are many attacks currently. We struggle to solve it now. You could construct the Dystopia's network easily. See [here](https://github.com/tbrand/dystopia/wiki/Getting-Started) for details.**

```bash
curl https://google.com -x 54.95.171.65:2888 -L
```

The curl execution through our public Dystopia's nodes. Nobody knows the exact route except the public gateway.

<i>Dystopia is on super super super early stage. Any feedbacks or contributions are very welcome!</i>

## Documents

### For Users
- [How does Dystopia work?](https://github.com/tbrand/dystopia/wiki/How-does-Dystopia-work%3F)
- [Performance Evaluation](https://github.com/tbrand/dystopia/wiki/Performance-Evaluation)
- [Launch your gateway](https://github.com/tbrand/dystopia/wiki/Launch-your-gateway)
- [Component: Gateway](https://github.com/tbrand/dystopia/wiki/Component:-Gateway)
- [Component: Node](https://github.com/tbrand/dystopia/wiki/Component:-Node)
- [Component: Cloud](https://github.com/tbrand/dystopia/wiki/Component:-Cloud)
- [Fault Tolerant](https://github.com/tbrand/dystopia/wiki/Fault-Tolerant)
- [Joining to our public cloud](https://github.com/tbrand/dystopia/wiki/Joining-to-our-public-cloud)

### For Developers
- [Getting Started](https://github.com/tbrand/dystopia/wiki/Getting-Started)
- [Build and Install](https://github.com/tbrand/dystopia/wiki/Build-and-Install)
- [Use docker image](https://github.com/tbrand/dystopia/wiki/Use-docker-image)
- [Testing](https://github.com/tbrand/dystopia/wiki/Testing)

## Appreciate your contributions!
- We need more nodes to acquire anonymity. Please [Joining to our public cloud](https://github.com/tbrand/dystopia/wiki/Joining-to-our-public-cloud) if you have any idle computing resources.
- We need to implemenet more to make the protduct robust and fast. Send pull requests or opening issues when you have proposals.
