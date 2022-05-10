<a name="unreleased"></a>
## [Unreleased]


<a name="v1.4.1"></a>
## [v1.4.1] - 2022-05-09
### Bug Fixes
- Relax CIP15 requirements and log level ([#290](https://github.com/txpipe/oura/issues/290))


<a name="v1.4.0"></a>
## [v1.4.0] - 2022-05-09
### Features
- Emit witness events ([#262](https://github.com/txpipe/oura/issues/262))
- Introduce GCP PubSub sink ([#237](https://github.com/txpipe/oura/issues/237))
- Add native scripts ([#241](https://github.com/txpipe/oura/issues/241))
- Add custom terminal format for ADA Handle assets ([#232](https://github.com/txpipe/oura/issues/232))
- Add option to include tx details in block events ([#231](https://github.com/txpipe/oura/issues/231))
- **CIP15:** Add CIP-0015 parser ([#124](https://github.com/txpipe/oura/issues/124))

### Doc
- Add [source.finalize] doc and example ([#258](https://github.com/txpipe/oura/issues/258))

### Bug Fixes
- missing fields in NativeScript fingerprint ([#246](https://github.com/txpipe/oura/issues/246))
- Ensure aws feature builds ok in isolation ([#230](https://github.com/txpipe/oura/issues/230))

### Docs
- Add automated changelog ([#286](https://github.com/txpipe/oura/issues/286))
- Fix typo in CONTRIBUTING.md ([#259](https://github.com/txpipe/oura/issues/259))
- Fix typo in README ([#239](https://github.com/txpipe/oura/issues/239))
- Fixed typos ([#226](https://github.com/txpipe/oura/issues/226))

### Continuous Integration
- Fix github / kubectl / eks issue
- Workaround github / kubectl / eks issue

### Chore
- Add min-depth to e2e tests ([#272](https://github.com/txpipe/oura/issues/272))
- Fix linting issues ([#244](https://github.com/txpipe/oura/issues/244))
- **deps:** bump log from 0.4.16 to 0.4.17 ([#285](https://github.com/txpipe/oura/issues/285))
- **deps:** bump serde from 1.0.136 to 1.0.137 ([#275](https://github.com/txpipe/oura/issues/275))
- **deps:** bump tokio from 1.18.0 to 1.18.1 ([#278](https://github.com/txpipe/oura/issues/278))
- **deps:** bump openssl from 0.10.38 to 0.10.40 ([#282](https://github.com/txpipe/oura/issues/282))
- **deps:** bump clap from 3.1.13 to 3.1.16 ([#283](https://github.com/txpipe/oura/issues/283))
- **deps:** bump serde_json from 1.0.79 to 1.0.81 ([#281](https://github.com/txpipe/oura/issues/281))
- **deps:** Update all aws sdk libs to 0.11 ([#271](https://github.com/txpipe/oura/issues/271))
- **deps:** bump clap from 3.1.12 to 3.1.13 ([#270](https://github.com/txpipe/oura/issues/270))
- **deps:** bump pallas from 0.9.0-alpha.0 to 0.9.0-alpha.1 ([#269](https://github.com/txpipe/oura/issues/269))
- **deps:** bump tokio from 1.17.0 to 1.18.0 ([#266](https://github.com/txpipe/oura/issues/266))
- **deps:** Remove unused TUI dependency ([#268](https://github.com/txpipe/oura/issues/268))
- **deps:** bump clap from 3.1.11 to 3.1.12 ([#261](https://github.com/txpipe/oura/issues/261))
- **deps:** bump config from 0.13.0 to 0.13.1 ([#248](https://github.com/txpipe/oura/issues/248))
- **deps:** bump clap from 3.1.8 to 3.1.11 ([#257](https://github.com/txpipe/oura/issues/257))
- **deps:** bump crossterm from 0.23.1 to 0.23.2 ([#235](https://github.com/txpipe/oura/issues/235))
- **deps:** bump clap from 3.1.16 to 3.1.17 ([#284](https://github.com/txpipe/oura/issues/284))
- **deps:** Update Pallas to version 0.8.0 ([#242](https://github.com/txpipe/oura/issues/242))
- **deps:** bump config from 0.12.0 to 0.13.0 ([#236](https://github.com/txpipe/oura/issues/236))
- **deps:** bump clap from 3.1.7 to 3.1.8 ([#234](https://github.com/txpipe/oura/issues/234))
- **deps:** bump clap from 3.1.6 to 3.1.7 ([#229](https://github.com/txpipe/oura/issues/229))
- **deps:** bump log from 0.4.14 to 0.4.16 ([#217](https://github.com/txpipe/oura/issues/217))


<a name="v1.3.2"></a>
## [v1.3.2] - 2022-04-26
### Bug Fixes
- Upgrade Pallas to deal with uint hashes


<a name="v1.3.1"></a>
## [v1.3.1] - 2022-04-16
### Bug Fixes
- Update Pallas to deal with metadata overflows

### Chore
- Fix lint warnings


<a name="v1.3.0"></a>
## [v1.3.0] - 2022-03-25
### Features
- Add epoch and epoch slot values to Block record ([#195](https://github.com/txpipe/oura/issues/195))
- Introduce AWS S3 sink ([#221](https://github.com/txpipe/oura/issues/221))
- Graceful shutdown options ([#216](https://github.com/txpipe/oura/issues/216))
- Add connection-retry logic with exponential backoff ([#213](https://github.com/txpipe/oura/issues/213))
- CLI option to override configured cursor in daemon mode ([#212](https://github.com/txpipe/oura/issues/212))
- Introduce AWS Lambda sink ([#208](https://github.com/txpipe/oura/issues/208))
- Introduce AWS SQS Sink ([#207](https://github.com/txpipe/oura/issues/207))

### Bug Fixes
- Hotfix release by skipping arm64 container build
- Implement missing S3 object naming conventions ([#223](https://github.com/txpipe/oura/issues/223))
- Hide SQS sink under correct feature flag ([#214](https://github.com/txpipe/oura/issues/214))

### Docs
- Fix missing AWS sinks in mdbook index ([#225](https://github.com/txpipe/oura/issues/225))
- Document AWS Sinks ([#224](https://github.com/txpipe/oura/issues/224))

### Code Refactoring
- Move sub-command definition to corresponding module ([#209](https://github.com/txpipe/oura/issues/209))

### Continuous Integration
- Add AWS e2e tests ([#222](https://github.com/txpipe/oura/issues/222))
- Add more e2e tests ([#219](https://github.com/txpipe/oura/issues/219))
- Introduce e2e testing workflow ([#218](https://github.com/txpipe/oura/issues/218))

### Chore
- **deps:** bump aws-sdk-sqs from 0.8.0 to 0.9.0 ([#211](https://github.com/txpipe/oura/issues/211))


<a name="v1.2.2"></a>
## [v1.2.2] - 2022-03-16
### Bug Fixes
- Downgrade metadata key issues to warnings ([#199](https://github.com/txpipe/oura/issues/199))

### Chore
- **deps:** Update Pallas with fix for payload regression ([#205](https://github.com/txpipe/oura/issues/205))
- **deps:** bump reqwest from 0.11.9 to 0.11.10 ([#200](https://github.com/txpipe/oura/issues/200))
- **deps:** bump crossterm from 0.23.0 to 0.23.1 ([#204](https://github.com/txpipe/oura/issues/204))
- **deps:** Upgrade Pallas to v0.7.0 ([#198](https://github.com/txpipe/oura/issues/198))
- **deps:** bump clap from 3.1.5 to 3.1.6 ([#188](https://github.com/txpipe/oura/issues/188))
- **deps:** Update pallas-primitives to v0.6.4 ([#191](https://github.com/txpipe/oura/issues/191))


<a name="v1.2.1"></a>
## [v1.2.1] - 2022-03-08
### Bug Fixes
- Fix testnet well-known time parameters ([#189](https://github.com/txpipe/oura/issues/189))

### Docs
- Add examples of complex selection filters ([#185](https://github.com/txpipe/oura/issues/185))

### Chore
- Use v1.2 for testdrive examples ([#182](https://github.com/txpipe/oura/issues/182))
- **deps:** Update pallas-primitives to v0.6.3 ([#190](https://github.com/txpipe/oura/issues/190))
- **deps:** bump strum from 0.23.0 to 0.24.0 ([#161](https://github.com/txpipe/oura/issues/161))
- **deps:** bump clap from 3.1.3 to 3.1.5 ([#186](https://github.com/txpipe/oura/issues/186))


<a name="v1.2.0"></a>
## [v1.2.0] - 2022-03-01
### Features
- Crawl Byron's epoch boundary blocks ([#169](https://github.com/txpipe/oura/issues/169))
- Introduce 'intersect' argument ([#165](https://github.com/txpipe/oura/issues/165))
- Implement Prometheus metric exporter ([#154](https://github.com/txpipe/oura/issues/154))
- Implement multi-era timestamp calculation ([#155](https://github.com/txpipe/oura/issues/155))
- Implement rollback buffer ([#149](https://github.com/txpipe/oura/issues/149))
- Introduce the 'Assert' sink ([#140](https://github.com/txpipe/oura/issues/140))
- Handle Byron blocks ([#138](https://github.com/txpipe/oura/issues/138))
- update the docs for the mapper config for the cbor change ([#137](https://github.com/txpipe/oura/issues/137))
- Add option to include raw block cbor ([#127](https://github.com/txpipe/oura/issues/127))

### Bug Fixes
- Pin dockerfile to "buster" Debian and update testdrive envs ([#181](https://github.com/txpipe/oura/issues/181))
- Downgrade all CIP-25 parser errors to warnings ([#180](https://github.com/txpipe/oura/issues/180))
- Downgrade 721 metadata error to warning ([#175](https://github.com/txpipe/oura/issues/175))
- Use magic from source in daemon bootstrap ([#166](https://github.com/txpipe/oura/issues/166))
- Compute timestamp in Byron mappings ([#156](https://github.com/txpipe/oura/issues/156))
- **fingerprint:** Passthrough events even on error ([#142](https://github.com/txpipe/oura/issues/142))

### Docs
- Document new features in v1.2 ([#171](https://github.com/txpipe/oura/issues/171))

### Code Refactoring
- Merge epoch boundary record with standard block ([#172](https://github.com/txpipe/oura/issues/172))

### Build
- **deps:** config-0.12.0 and fixes ([#178](https://github.com/txpipe/oura/issues/178))
- **deps:** clap-3.1.3 and fixes ([#179](https://github.com/txpipe/oura/issues/179))

### Continuous Integration
- Remove i686 release targets ([#129](https://github.com/txpipe/oura/issues/129))
- **dependabot:** Auto-update GH Action versions ([#130](https://github.com/txpipe/oura/issues/130))

### Chore
- Update Pallas to version 0.5.0-alpha.3 ([#153](https://github.com/txpipe/oura/issues/153))
- Add cursor to testdrive examples ([#139](https://github.com/txpipe/oura/issues/139))
- Update testdrive scripts to latest version
- Update pallas-primitives to v0.5.3
- Update Pallas primitives version ([#168](https://github.com/txpipe/oura/issues/168))
- Update Pallas miniprotocols 0.5.1 ([#167](https://github.com/txpipe/oura/issues/167))
- Upgrade to Pallas 0.5.0-alpha.1 ([#148](https://github.com/txpipe/oura/issues/148))
- **deps:** Update pallas-primitives to v0.6.2 ([#177](https://github.com/txpipe/oura/issues/177))
- **deps:** bump tokio from 1.16.1 to 1.17.0 ([#151](https://github.com/txpipe/oura/issues/151))
- **deps:** bump serde_json from 1.0.78 to 1.0.79 ([#141](https://github.com/txpipe/oura/issues/141))
- **deps:** bump strum_macros from 0.23.1 to 0.24.0 ([#159](https://github.com/txpipe/oura/issues/159))
- **deps:** Update Pallas to v0.6 (includes minicbor 0.14) ([#173](https://github.com/txpipe/oura/issues/173))
- **deps:** Update pallas-primitives to v0.6.1 ([#174](https://github.com/txpipe/oura/issues/174))
- **deps:** bump clap from 3.0.13 to 3.0.14 ([#121](https://github.com/txpipe/oura/issues/121))
- **deps:** bump file-rotate from 0.5.3 to 0.6.0 ([#133](https://github.com/txpipe/oura/issues/133))
- **deps:** bump minicbor from 0.13.1 to 0.13.2 ([#134](https://github.com/txpipe/oura/issues/134))
- **deps:** bump crossterm from 0.22.1 to 0.23.0 ([#135](https://github.com/txpipe/oura/issues/135))
- **deps:** bump minicbor from 0.13.0 to 0.13.1 ([#125](https://github.com/txpipe/oura/issues/125))


<a name="v1.1.0"></a>
## [v1.1.0] - 2022-02-05
### Features
- Introduce stateful cursor ([#116](https://github.com/txpipe/oura/issues/116))
- Introduce 'stoud' + 'logs' sink ([#77](https://github.com/txpipe/oura/issues/77))
- **BlockRecord:** include previous block hash ([#120](https://github.com/txpipe/oura/issues/120))
- **model:** include tx_hash in TransactionRecord ([#123](https://github.com/txpipe/oura/issues/123))
- **watch:** Add output throttle cli arg

### Bug Fixes
- EventWriter::standalone() inaccessible ([#115](https://github.com/txpipe/oura/issues/115))
- Make bech32 encoding network-aware ([#104](https://github.com/txpipe/oura/issues/104))
- Slot to timestamp mapping matches public explorers ([#101](https://github.com/txpipe/oura/issues/101))
- dump build without 'logs' feature ([#82](https://github.com/txpipe/oura/issues/82))

### Docs
- Add documentation for new v1.1 features ([#126](https://github.com/txpipe/oura/issues/126))
- Add contributing guide ([#83](https://github.com/txpipe/oura/issues/83))

### Code Refactoring
- Streamline access shared utilities ([#108](https://github.com/txpipe/oura/issues/108))

### Continuous Integration
- Start linting both code and some support files ([#96](https://github.com/txpipe/oura/issues/96))
- Add testdrive for logs sink ([#98](https://github.com/txpipe/oura/issues/98))
- **style:** Add EditorConfig and relevant GH Action ([#91](https://github.com/txpipe/oura/issues/91))

### Chore
- remove explicit of 'use serde_derive'
- **deps:** Use Pallas 0.4.0 ([#118](https://github.com/txpipe/oura/issues/118))
- **deps:** bump clap from 3.0.12 to 3.0.13 ([#105](https://github.com/txpipe/oura/issues/105))
- **deps:** bump clap from 3.0.10 to 3.0.12 ([#99](https://github.com/txpipe/oura/issues/99))
- **deps:** bump serde from 1.0.134 to 1.0.135 ([#89](https://github.com/txpipe/oura/issues/89))
- **deps:** bump tui from 0.16.0 to 0.17.0 ([#90](https://github.com/txpipe/oura/issues/90))
- **deps:** bump serde_json from 1.0.75 to 1.0.78 ([#88](https://github.com/txpipe/oura/issues/88))
- **deps:** bump serde from 1.0.133 to 1.0.134
- **deps:** bump serde from 1.0.135 to 1.0.136 ([#102](https://github.com/txpipe/oura/issues/102))
- **deps:** bump clap from 3.0.9 to 3.0.10
- **deps:** bump clap from 3.0.7 to 3.0.9


<a name="v1.0.2"></a>
## [v1.0.2] - 2022-01-17
### Bug Fixes
- Use json-compatible structure for MoveInstantaneousRewardsCert ([#71](https://github.com/txpipe/oura/issues/71))
- **mapper:** Panic on inter-stage channel error ([#70](https://github.com/txpipe/oura/issues/70))

### Chore
- **deps:** bump serde_json from 1.0.74 to 1.0.75


<a name="v1.0.1"></a>
## [v1.0.1] - 2022-01-15
### Bug Fixes
- **terminal:** Avoid slicing utf-8 chars ([#68](https://github.com/txpipe/oura/issues/68))

### Docs
- Use v1 for docker example ([#64](https://github.com/txpipe/oura/issues/64))
- Add missing entry to summary
- **webhook:** Fix webhook testdrive config ([#63](https://github.com/txpipe/oura/issues/63))


<a name="v1.0.0"></a>
## [v1.0.0] - 2022-01-13
### Features
- Add 'end' events for blocks and txs ([#60](https://github.com/txpipe/oura/issues/60))
- Add slot, hash and number to block start event ([#59](https://github.com/txpipe/oura/issues/59))
- Introduce "Webhook" sink ([#51](https://github.com/txpipe/oura/issues/51))
- Add CIP-25 metadata parser ([#49](https://github.com/txpipe/oura/issues/49))
- Auto-detect version
- **mapper:** Refactor event-mapper code for easier extension ([#47](https://github.com/txpipe/oura/issues/47))

### Bug Fixes
- Log & continue on mapper errors ([#53](https://github.com/txpipe/oura/issues/53))
- CIP25 json key naming
- Remove rogue println

### Docs
- Add testdrive example for webhook setup ([#55](https://github.com/txpipe/oura/issues/55))
- Add testdrive example for Elasticsearch setup ([#54](https://github.com/txpipe/oura/issues/54))
- **webhook:** Add webhook sink configuration docs ([#62](https://github.com/txpipe/oura/issues/62))

### Style
- Fix whitespaces

### Build
- Enable dependabot

### Chore
- Prep for v1 ([#58](https://github.com/txpipe/oura/issues/58))
- Fix lint warnings
- **deps:** bump reqwest from 0.11.8 to 0.11.9
- **deps:** bump clap from 3.0.6 to 3.0.7
- **deps:** bump clap from 3.0.5 to 3.0.6
- **deps:** Update Pallas to version 3.9 ([#44](https://github.com/txpipe/oura/issues/44))
- **deps:** Bump-up pallas v0.3.3 to v0.3.4


<a name="v0.3.10"></a>
## [v0.3.10] - 2022-01-08
### Features
- **watch:** Show error logs in stdout by default ([#41](https://github.com/txpipe/oura/issues/41))

### Chore
- **deps:** Update Pallas to version 0.3.8 ([#38](https://github.com/txpipe/oura/issues/38))


<a name="v0.3.9"></a>
## [v0.3.9] - 2022-01-07
### Features
- Use sync channel for n2n intra-stage messaging
- Switch to sync std mpsc channels
- Centralize inter-stage channel setup
- Basic Windows support ([#20](https://github.com/txpipe/oura/issues/20))
- **cli:** Add version number to help output

### Bug Fixes
- Typo in watch arg parsing
- Move from value_t macro to ArgMatches::value_of_t
- Update more code to new Clap API
- Clap API update

### Docs
- Improve documentation across the board

### Style
- Fix formatting
- Fix formatting
- Fix lints

### Chore
- **deps:** Update Pallas version and other patched dependencies
- **deps:** bump clap from 2.34.0 to 3.0.5
- **deps:** Update pallas to v0.3.5 ([#30](https://github.com/txpipe/oura/issues/30))
- **deps:** bump serde_json from 1.0.73 to 1.0.74
- **deps:** bump serde from 1.0.132 to 1.0.133
- **deps:** bump minicbor from 0.12.0 to 0.12.1
- **deps:** bump serde_json from 1.0.72 to 1.0.73 ([#23](https://github.com/txpipe/oura/issues/23))
- **deps:** bump crossterm from 0.20.0 to 0.22.1 ([#22](https://github.com/txpipe/oura/issues/22))
- **deps:** bump serde from 1.0.130 to 1.0.132 ([#21](https://github.com/txpipe/oura/issues/21))

### BREAKING CHANGE

Metadata record presents new structure.

Level of granularity for metadata event is different, one record per label.

Configuration keys for the 'Selection' filter changed to reflect new metadata structure


<a name="v0.3.8"></a>
## [v0.3.8] - 2021-12-30
### Features
- Add details to 'transaction' event ([#24](https://github.com/txpipe/oura/issues/24))
- Introduce 'selection' filter ([#18](https://github.com/txpipe/oura/issues/18))
- Introduce 'fingerprint' filter ([#16](https://github.com/txpipe/oura/issues/16))
- Introduce 'filtering' stage ([#14](https://github.com/txpipe/oura/issues/14))
- Add common data aggregations to events ([#13](https://github.com/txpipe/oura/issues/13))
- **node:** Allow reading from arbitrary initial chain point ([#10](https://github.com/txpipe/oura/issues/10))
- **node:** Output block hash as event data

### Bug Fixes
- **node:** Handle non-map metadata structures ([#12](https://github.com/txpipe/oura/issues/12))

### Docs
- Add fingerprint / selection filter docs
- Improve 'watch' mode usage info
- Move docker instructions to mdbook

### Style
- Fix whitespaces ([#17](https://github.com/txpipe/oura/issues/17))

### Continuous Integration
- Add 'testdrive' workflow
- fix branch filter for validate workflow

### Chore
- Update README feature check list
- Fix clippy / fmt warnings
- Fix fmt / clippy warnings
- Fix fmt / clippy warnings
- **deps:** Bump-up pallas v0.3.3 to v0.3.4 ([#19](https://github.com/txpipe/oura/issues/19))


<a name="v0.3.7"></a>
## [v0.3.7] - 2021-12-23
### Features
- **elastic:** Improve Elasticsearch sink implementation ([#8](https://github.com/txpipe/oura/issues/8))

### Bug Fixes
- Allow env override of nested configs

### Chore
- Fix fmt and clippy warnings


<a name="v0.3.6"></a>
## [v0.3.6] - 2021-12-21
### Features
- Compute slot timestamp ([#6](https://github.com/txpipe/oura/issues/6))

### Continuous Integration
- Improve docker build speed ([#7](https://github.com/txpipe/oura/issues/7))

### Chore
- Tidy up CI workflows


<a name="v0.3.5-docker1"></a>
## [v0.3.5-docker1] - 2021-12-19
### Bug Fixes
- Include all features in docker build

### Docs
- Link to Kafka's original site ([#4](https://github.com/txpipe/oura/issues/4))


<a name="v0.3.5"></a>
## [v0.3.5] - 2021-12-18
### Features
- include event type tag in serde output
- **elastic:** Add Elasticsearch MVP Sink ([#5](https://github.com/txpipe/oura/issues/5))

### Bug Fixes
- Add all features in the binary releases
- use correct cfg feature syntax
- valid entrypoint on Dockerfile

### Docs
- improve binary install example
- add context field to data dictionary
- fill data dictionary and installation info
- point readme to mdbook
- add draft documentation in mdbook format


<a name="v0.3.4"></a>
## [v0.3.4] - 2021-12-15
### Bug Fixes
- **n2c:** remove v10 version constraint


<a name="v0.3.3"></a>
## [v0.3.3] - 2021-12-14
### Bug Fixes
- use correct cursor on chainsync roll forward [#2](https://github.com/txpipe/oura/issues/2)


<a name="v0.3.2"></a>
## [v0.3.2] - 2021-12-13
### Docs
- add cardano to kafka example

### Code Refactoring
- migrate to new version of pallas-alonzo


<a name="v0.3.1-docker3"></a>
## [v0.3.1-docker3] - 2021-12-12
### Continuous Integration
- remove extra container platforms to speed up workflow
- add docker build to release workflow


<a name="v0.3.1-docker"></a>
## [v0.3.1-docker] - 2021-12-12
### Continuous Integration
- add docker build to release workflow


<a name="v0.3.1-docker2"></a>
## [v0.3.1-docker2] - 2021-12-12
### Continuous Integration
- add docker build to release workflow


<a name="v0.3.1"></a>
## [v0.3.1] - 2021-12-11
### Features
- represent addresses in bech32 format
- show rollback control event

### Bug Fixes
- remove local path dependencies

### Chore
- bump version patch
- apply lint suggestions / formatting
- remove unwraps and do a graceful exit
- **terminal:** tidy up console output


<a name="v0.3.0"></a>
## [v0.3.0] - 2021-12-10
### Features
- add certificate data mappings
- **sources:** implement node-to-node soure using chainsync + blockfetch
- **terminal:** improve output style

### Docs
- add cli instructions

### Style
- fix lint warnings

### Chore
- bump version minor
- improve source folder structure
- improve readme
- explain extra dep on cargo


<a name="v0.2.0"></a>
## [v0.2.0] - 2021-12-08
### Features
- compute and output tx hashes
- **daemon:** implement basic daemon cli subcommand
- **kafka:** add basic kafka sink implementation

### Chore
- fix missing openssl in release workflow
- apply cargo fmt
- fix missing openssl in release workflow
- bump version for release
- add missing info to cargo metadata
- fix license year typo
- fix lint issues
- fix lint issues
- improve args in log subcommand
- add use-cases to readme
- remove makefile
- **framework:** define bootstrap traits for components


<a name="v0.1.0"></a>
## v0.1.0 - 2021-12-05
### Continuous Integration
- add github workflows

### Chore
- fix lint issues
- tidy up arg parsing in oura bin
- add code of conduct
- fix lint issues


[Unreleased]: https://github.com/txpipe/oura/compare/v1.4.1...HEAD
[v1.4.1]: https://github.com/txpipe/oura/compare/v1.4.0...v1.4.1
[v1.4.0]: https://github.com/txpipe/oura/compare/v1.3.2...v1.4.0
[v1.3.2]: https://github.com/txpipe/oura/compare/v1.3.1...v1.3.2
[v1.3.1]: https://github.com/txpipe/oura/compare/v1.3.0...v1.3.1
[v1.3.0]: https://github.com/txpipe/oura/compare/v1.2.2...v1.3.0
[v1.2.2]: https://github.com/txpipe/oura/compare/v1.2.1...v1.2.2
[v1.2.1]: https://github.com/txpipe/oura/compare/v1.2.0...v1.2.1
[v1.2.0]: https://github.com/txpipe/oura/compare/v1.1.0...v1.2.0
[v1.1.0]: https://github.com/txpipe/oura/compare/v1.0.2...v1.1.0
[v1.0.2]: https://github.com/txpipe/oura/compare/v1.0.1...v1.0.2
[v1.0.1]: https://github.com/txpipe/oura/compare/v1.0.0...v1.0.1
[v1.0.0]: https://github.com/txpipe/oura/compare/v0.3.10...v1.0.0
[v0.3.10]: https://github.com/txpipe/oura/compare/v0.3.9...v0.3.10
[v0.3.9]: https://github.com/txpipe/oura/compare/v0.3.8...v0.3.9
[v0.3.8]: https://github.com/txpipe/oura/compare/v0.3.7...v0.3.8
[v0.3.7]: https://github.com/txpipe/oura/compare/v0.3.6...v0.3.7
[v0.3.6]: https://github.com/txpipe/oura/compare/v0.3.5-docker1...v0.3.6
[v0.3.5-docker1]: https://github.com/txpipe/oura/compare/v0.3.5...v0.3.5-docker1
[v0.3.5]: https://github.com/txpipe/oura/compare/v0.3.4...v0.3.5
[v0.3.4]: https://github.com/txpipe/oura/compare/v0.3.3...v0.3.4
[v0.3.3]: https://github.com/txpipe/oura/compare/v0.3.2...v0.3.3
[v0.3.2]: https://github.com/txpipe/oura/compare/v0.3.1-docker3...v0.3.2
[v0.3.1-docker3]: https://github.com/txpipe/oura/compare/v0.3.1-docker...v0.3.1-docker3
[v0.3.1-docker]: https://github.com/txpipe/oura/compare/v0.3.1-docker2...v0.3.1-docker
[v0.3.1-docker2]: https://github.com/txpipe/oura/compare/v0.3.1...v0.3.1-docker2
[v0.3.1]: https://github.com/txpipe/oura/compare/v0.3.0...v0.3.1
[v0.3.0]: https://github.com/txpipe/oura/compare/v0.2.0...v0.3.0
[v0.2.0]: https://github.com/txpipe/oura/compare/v0.1.0...v0.2.0
