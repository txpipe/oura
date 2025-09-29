# Changelog

All notable changes to this project will be documented in this file.

## [2.0.0] - 2025-09-29

### 🚀 Features

- Support all record types in terminal sink (#839)
- Support env filters for tracing output (#877)
- Introduce ZeroMQ sink (#872)
- Update mithril-client and aws dependencies (#897)

### 🐛 Bug Fixes

- Make select filter artifacs public for library mode (#892)
- Support reset events in rollback buffer (#894)

### 🚜 Refactor

- Make config struct fields public (#899)

### 📚 Documentation

- Improve utxorpc source section (#867)

### ⚙️ Miscellaneous Tasks

- Update Pallas to v0.33.0 (#875)
- Update readme for v2 release (#880)
- Update readme badges (#881)
- Tidy up release procedure

## [2.0.0-beta.0] - 2025-04-18

### 🚀 Features

- Make source and sink mods public to use as library (#806)
- Use block data for u5c (#832)
- *(u5c)* Allow arbitrary request metadata (#833)
- Add reference_inputs to TransactionRecord (#810)
- Add hydra source (#823)
- Implement rollback filter (#855)
- Implement dump command (#853)
- Added cargo dist (#850)
- Implement watch command (#852)

### 🐛 Bug Fixes

- *(select)* Handle payment-part address matching (#818)
- *(parse_cbor)* Support mapping of CborBlock to ParsedBlock (#824)
- *(redis)* Update deps and fix deprecations (#831)

### 💼 Other

- Bump to version 2.0.0-beta.0 (#863)

### 🚜 Refactor

- Factor out `run_daemon` and move to library (#829)

### 📚 Documentation

- Updated discord link (#846)
- Added wasm filter and removed deno (#851)
- Add example using Oura as a library (#854)
- Update docs to new astro pattern (#857)

### ⚙️ Miscellaneous Tasks

- Update Pallas to git edge (#830)
- Fix lint warnings (#847)
- Add paulobressan as code owner (#856)
- Add gonzalezzfelipe as code owner (#858)
- Remove deprecated `include_byron_ebb` option (#860)
- Update u5c crate to v0.10 (#859)
- Add homebrew installer to release (#861)
- Fix lint warnings (#862)
- Use supported runners for release workflow (#864)
- Define release runners  (#865)

## [2.0.0-alpha.4] - 2024-09-08

### 🐛 Bug Fixes

- Use complete u5c interop mapping (#816)

## [2.0.0-alpha.3] - 2024-08-28

### 🚀 Features

- Add filtering by witnesses (#544)
- Introduce RabbitMQ sink (#550)
- Migrate to new SEDA-based sources (#481)
- Implement legacy v1 mapper (#554)
- Include noop sink
- Introduce Deno mapper stage (#560)
- *(deno)* Allow async mapper option (#567)
- Add cbor-parsing filter (#578)
- Add raw-cbor S3 source (#587)
- Allow well-known network by name (#606)
- Use specific names for stages (#640)
- *(redis)* Allow capping stream size (#636)
- Implement utxorpc source (#664)
- Implement scaffold for match pattern filter (#676)
- Implement file-based chain cursor (#723)
- Introduce sql db sink (#744)
- Enable PostgreSQL engine for sql sink (#745)
- Overhaul the selection filter (#729)
- Introduce wasm filter using Extism (#761)
- Implement `IntoJson` filter (#768)
- Implement Redis option for cursor persistence (#790)
- Implement Mithril source stage (#795)

### 🐛 Bug Fixes

- *(rabbitmq)* Exit process if connection closes unexpectedly
- Track latest block in sink stages (#572)
- Show legacy logs as tracing (#601)
- Relax stage runtime policy (#602)
- *(deno)* Allow big integers in payloads (#607)
- Honor transaction details flag without requiring block details flag (#673)
- Include Deno filter only when feature flag is on (#698)
- Fix accumulated clippy warnings (#700)
- Remove hardcoded stage tick timeout (#705)
- Use new aws config mechanism (#727)
- *(legacy)* Use custom json serde for i128 (#750)
- Apply unix flag to n2c source (#756)

### 🚜 Refactor

- Migrate to async workers (#577)
- Revisit feature flag naming & grouping (#777)
- Rename utxorpc source to u5c (#778)

### 📚 Documentation

- Improve docs across the board (#553)
- Add execution steps to cip68 example
- Add file-rotate filter example (#570)
- Add pool-metadata example (#571)
- Improve basic Deno example (#579)
- Update readme for V2 (#581)
- Update deno_cip68 example readme (#611)
- Update docs regarding retry configuration
- Improve sinks docs regarding retry configs (#644)
- Migrate docs to Nextra (#654)
- Improve documentation across the board (#669)
- Fix typo in utxorpc documentation (#677)
- UI for building configs (#684)
- Improve config generator (#696)

### ⚙️ Miscellaneous Tasks

- Re-organize mapper module structure (#482)
- Refactor bin entry point (#483)
- Start migration to gasket framework (#493)
- Migrate new gasket plexer (#500)
- Migrate to new SEDA-based chainsync stage (#501)
- Migrate blockfetch stage to new SEDA version (#502)
- Connect minimal v2 pipeline
- Connect full v2 pipeline (#557)
- Unify ops count metric
- Merge filters and mappers into a single stage (#561)
- Add CIP68 Deno example (#562)
- Include Deno utils as asset (#565)
- Migrate file-rotate filter to v2 pipeline (#569)
- Remove legacy gcp support files
- Migrate webhook to new pipeline (#580)
- Update Pallas / Gasket deps (#584)
- Use u5c from crates.io (#586)
- Upgrade gasket to v0.3.0 (#590)
- Upgrade gasket to v0.4 (#593)
- Migrate n2c source to new pipeline (#598)
- Migrate stdout sink to new pipeline (#600)
- Update Pallas to v0.19.0-alpha.1 (#610)
- Add feature flag for webhook sink (#615)
- *(rabbitmq)* Migrate RabbitMQ sink to new pipeline (#614)
- Use `sink-` prefix feature flags (#620)
- Migrate aws sqs sink to new pipeline (#622)
- Migrate kafka sink to new pipeline (#621)
- Migrate aws lambda sink to new pipeline (#623)
- Migrate Redis sink to new pipeline (#629)
- Migrate gcp pubsub sink to new pipeline (#624)
- Migrate Elasticsearch sink to new pipeline (#631)
- Migrate GCP cloud-function sink (#634)
- Adjust file rotate sink to use features (#649)
- Migrate S3 sink to new pipeline (#646)
- Migrate assert sink to new pipeline (#652)
- Improve docs SEO (#674)
- Update Deno dependencies (#689)
- Update deno dependences (#692)
- Include protoc dependency on github workflows (#699)
- Restore prometheus metrics in v2 (#728)
- Upgrade Pallas to v0.21 (#743)
- Update nix flake (#760)
- Remove legacy selection filter (#766)
- Remove legacy testdrive configs (#767)
- Deprecate Deno in favor of Wasm (#779)
- Migrate to gasket prometheus exporter (#780)
- Remove legacy code (#781)
- Add manual trigger for testdrive workflow (#789)
- Update Pallas to v0.27 (#791)
- Update OCI base image to Debian 12 (bookworm) (#804)
- Upgrade Pallas to v0.30.1 (#812)

## [1.8.1] - 2023-02-04

### 🚀 Features

- Send inline datum as new events (#539)

### 🐛 Bug Fixes

- Use original cbor to define inline datum hash (#538)
- Use correct bytes for Byron addresses (#537)

### 📚 Documentation

- Add missing GCP PubSub item to index (#534)

### ⚙️ Miscellaneous Tasks

- Fix build badge (#533)

## [1.8.0] - 2023-01-30

### 🚀 Features

- Expose collateral data (#495)
- Add vrf_key to block event data (#489)

### 🐛 Bug Fixes

- Evaluate CIP25 policy / asset in selection filter (#498)
- *(gcp)* Switch to pubsub lib that handles token refresh (#512)
- Fix time calculation for preview / preprod (#528)
- Compute datum hash for inline values (#529)
- Fix byron address string representation (#530)

### 🚜 Refactor

- Switch to Pallas v0.17 (huge change) (#527)

### 📚 Documentation

- Fix Transaction ID typo in data dictionary
- Fix typos across the board (#523)

### ⚙️ Miscellaneous Tasks

- Upgrade Debian base image in Dockerfile (#520)
- Fix remaining lint warnings (#531)

## [1.7.3] - 2022-11-16

### 🐛 Bug Fixes

- Bump Pallas to fix Plutus data issue (#469)

### ⚙️ Miscellaneous Tasks

- Update broken e2e tests (#470)

## [1.7.2] - 2022-10-19

### 🐛 Bug Fixes

- Upgrade Pallas to fix CBOR issue (#460)

### 📚 Documentation

- Fix small typo in proposed filename (#448)

## [1.7.1] - 2022-09-13

### 🐛 Bug Fixes

- Apply missing selection filters at Tx level (#430)
- *(terminal)* Be aware of UTF-8 chars when truncating output (#431)

## [1.7.0] - 2022-09-11

### 🚀 Features

- Implement selection filter by Address (#396)
- Add cardano2dgraph testdrive example (#395)
- Add transaction size value to TransactionRecord (#403)
- *(terminal)* Allow user-defined terminal width (#393)

### 🐛 Bug Fixes

- Evaluate Tx records for metadata filters (#406)
- Fix typo in try_from_magic error output (#405)
- Fix incorrect error message in N2C stage (#402)
- Fix lint warning across the board  (#410)
- *(logs)* Fix log sink for non-unix targets (#425)

### 📚 Documentation

- Improve mapper options docs (#400)
- Fix typos and improve grammar in `selection` docs (#399)
- Add preview and preprod magic values to `watch` usage (#398)

### ⚙️ Miscellaneous Tasks

- *(terminal)* Use 'wrap' semantics for terminal width (#426)

## [1.6.0] - 2022-08-20

### 🚀 Features

- Add shortcuts for 'preview' and 'pre-prod' networks (#385)
- *(webhook)* Allow self-signed certificates (#390)

### 🐛 Bug Fixes

- *(elastic)* Don't panic on ID conflicts (#391)

### 📚 Documentation

- Describe 'retry policy' mechanism (#392)

### ⚙️ Miscellaneous Tasks

- Fix formatting issues (#388)

## [1.5.2] - 2022-08-02

### 🐛 Bug Fixes

- Fix JSON serialization of genesis key delegation (#372)

## [1.5.1] - 2022-07-04

### 🚀 Features

- Allow json & yaml as configuration file formats (#347)
- Remove e1 prefix from reward account (#379)

### 🐛 Bug Fixes

- Honor config field that toggles compression (#358)
- Move cursor only after side-effect (#364)
- Fix n2n babbage header parsing (#355)

### 📚 Documentation

- Add custom network config instructions (#362)
- Add redis streams documentation to book index (#363)
- Expand Redis sink section (#366)
- Update metadata-based selection predicates (#380)

### ⚙️ Miscellaneous Tasks

- Add all features flag in from source docs (#377)

## [1.5.0] - 2022-07-03

### 🚀 Features

- Retry whole chainsync operation when possible (#332)
- Add a nix flake (#335)
- Add metadata hash to pool registration event (#336)
- Implement Babbage compatibility (#351)

### 🐛 Bug Fixes

- Accommodate partial features build (#333)
- Add default values for retry policies (#352)
- Decode block wrappers correctly (#353)

### 🚜 Refactor

- Unify retry mechanism across sinks (#302)

## [1.4.3] - 2022-06-08

### 🐛 Bug Fixes

- Allow integer values in magic args (#320)
- Add missing details in tx record (#321)

### 📚 Documentation

- Update changelog

## [1.4.2] - 2022-06-05

### 🐛 Bug Fixes

- Upgrade Pallas to fix tx hash mismatch (#312)
- Add missing finalize option to N2C (#314)
- Include EBB blocks in E2E tests (#315)

### ⚙️ Miscellaneous Tasks

- Add N2C E2E tests (#313)

## [1.4.1] - 2022-05-10

### 🚀 Features

- Introduce Redis Streams sink (#253)

### 🐛 Bug Fixes

- Relax CIP15 requirements and log level (#290)

### 📚 Documentation

- Fix typo in daemon example (#294)
- Improve "data dictionary" section (#297)
- Add guide on connecting to custom networks (#306)

### ⚙️ Miscellaneous Tasks

- Fix lint warnings across the board (#310)

## [1.4.0] - 2022-05-09

### 🚀 Features

- Emit witness events (#262)
- *(CIP15)* Add CIP-0015 parser (#124)

### 📚 Documentation

- Add automated changelog (#286)

### ⚙️ Miscellaneous Tasks

- Add min-depth to e2e tests (#272)
- Workaround github / kubectl / eks issue
- Fix github / kubectl / eks issue

## [1.3.2] - 2022-04-26

### 🐛 Bug Fixes

- Upgrade Pallas to deal with uint hashes

## [1.3.1] - 2022-04-16

### 🚀 Features

- Add option to include tx details in block events (#231)
- Add custom terminal format for ADA Handle assets (#232)
- Add native scripts (#241)
- Introduce GCP PubSub sink (#237)

### 🐛 Bug Fixes

- Ensure aws feature builds ok in isolation (#230)
- Missing fields in NativeScript fingerprint (#246)
- Update Pallas to deal with metadata overflows

### 📚 Documentation

- Fixed typos (#226)
- Fix typo in README (#239)
- Add [source.finalize] doc and example (#258)
- Fix typo in CONTRIBUTING.md (#259)

### ⚙️ Miscellaneous Tasks

- Fix linting issues (#244)
- Fix lint warnings

## [1.3.0] - 2022-03-25

### 🚀 Features

- Introduce AWS SQS Sink (#207)
- Introduce AWS Lambda sink (#208)
- CLI option to override configured cursor in daemon mode (#212)
- Add connection-retry logic with exponential backoff (#213)
- Graceful shutdown options (#216)
- Introduce AWS S3 sink (#221)
- Add epoch and epoch slot values to Block record (#195)

### 🐛 Bug Fixes

- Hide SQS sink under correct feature flag (#214)
- Implement missing S3 object naming conventions (#223)
- Hotfix release by skipping arm64 container build

### 🚜 Refactor

- Move sub-command definition to corresponding module (#209)

### 📚 Documentation

- Document AWS Sinks (#224)
- Fix missing AWS sinks in mdbook index (#225)

### ⚙️ Miscellaneous Tasks

- Introduce e2e testing workflow (#218)
- Add more e2e tests (#219)
- Add AWS e2e tests (#222)

## [1.2.2] - 2022-03-16

### 🐛 Bug Fixes

- Downgrade metadata key issues to warnings (#199)

## [1.2.1] - 2022-03-08

### 🐛 Bug Fixes

- Fix testnet well-known time parameters (#189)

### 📚 Documentation

- Add examples of complex selection filters (#185)

### ⚙️ Miscellaneous Tasks

- Use v1.2 for testdrive examples (#182)

## [1.2.0] - 2022-03-01

### 🚀 Features

- Add option to include raw block cbor (#127)
- Update the docs for the mapper config for the cbor change (#137)
- Handle Byron blocks (#138)
- Introduce the 'Assert' sink (#140)
- Implement rollback buffer (#149)
- Implement multi-era timestamp calculation (#155)
- Implement Prometheus metric exporter (#154)
- Introduce 'intersect' argument (#165)
- Crawl Byron's epoch boundary blocks (#169)

### 🐛 Bug Fixes

- *(fingerprint)* Passthrough events even on error (#142)
- Compute timestamp in Byron mappings (#156)
- Use magic from source in daemon bootstrap (#166)
- Downgrade 721 metadata error to warning (#175)
- Downgrade all CIP-25 parser errors to warnings (#180)
- Pin dockerfile to "buster" Debian and update testdrive envs (#181)

### 💼 Other

- *(deps)* Clap-3.1.3 and fixes (#179)
- *(deps)* Config-0.12.0 and fixes (#178)

### 🚜 Refactor

- Merge epoch boundary record with standard block (#172)

### 📚 Documentation

- Document new features in v1.2 (#171)

### ⚙️ Miscellaneous Tasks

- Remove i686 release targets (#129)
- *(dependabot)* Auto-update GH Action versions (#130)
- Add cursor to testdrive examples (#139)
- Update testdrive scripts to latest version
- Upgrade to Pallas 0.5.0-alpha.1 (#148)
- Update Pallas to version 0.5.0-alpha.3 (#153)
- Update Pallas miniprotocols 0.5.1 (#167)
- Update Pallas primitives version (#168)
- Update pallas-primitives to v0.5.3

## [1.1.0] - 2022-02-05

### 🚀 Features

- *(watch)* Add output throttle cli arg
- Introduce 'stoud' + 'logs' sink (#77)
- *(BlockRecord)* Include previous block hash (#120)
- Introduce stateful cursor (#116)
- *(model)* Include tx_hash in TransactionRecord (#123)

### 🐛 Bug Fixes

- Dump build without 'logs' feature (#82)
- Slot to timestamp mapping matches public explorers (#101)
- Make bech32 encoding network-aware (#104)
- EventWriter::standalone() inaccessible (#115)

### 🚜 Refactor

- Streamline access shared utilities (#108)

### 📚 Documentation

- Add contributing guide (#83)
- Add documentation for new v1.1 features (#126)

### ⚙️ Miscellaneous Tasks

- Remove explicit of 'use serde_derive'
- *(style)* Add EditorConfig and relevant GH Action (#91)
- Add testdrive for logs sink (#98)
- Start linting both code and some support files (#96)

## [1.0.2] - 2022-01-18

### 🐛 Bug Fixes

- *(mapper)* Panic on inter-stage channel error (#70)
- Use json-compatible structure for MoveInstantaneousRewardsCert (#71)

## [1.0.1] - 2022-01-15

### 🐛 Bug Fixes

- *(terminal)* Avoid slicing utf-8 chars (#68)

### 📚 Documentation

- Add missing entry to summary
- *(webhook)* Fix webhook testdrive config (#63)
- Use v1 for docker example (#64)

## [1.0.0] - 2022-01-13

### 🚀 Features

- Auto-detect version
- *(mapper)* Refactor event-mapper code for easier extension (#47)
- Add CIP-25 metadata parser (#49)
- Introduce "Webhook" sink (#51)
- Add slot, hash and number to block start event (#59)
- Add 'end' events for blocks and txs (#60)

### 🐛 Bug Fixes

- Remove rogue println
- CIP25 json key naming
- Log & continue on mapper errors (#53)

### 📚 Documentation

- Add testdrive example for Elasticsearch setup (#54)
- Add testdrive example for webhook setup (#55)
- *(webhook)* Add webhook sink configuration docs (#62)

### ⚙️ Miscellaneous Tasks

- Fix lint warnings
- Prep for v1 (#58)

## [0.3.10] - 2022-01-08

### 🚀 Features

- *(watch)* Show error logs in stdout by default (#41)

## [0.3.9] - 2022-01-07

### 🚀 Features

- Basic Windows support (#20)
- [**breaking**] Map metadata as structured JSON (#29)
- Centralize inter-stage channel setup
- Switch to sync std mpsc channels
- Use sync channel for n2n intra-stage messaging
- *(cli)* Add version number to help output

### 🐛 Bug Fixes

- Clap API update
- Update more code to new Clap API
- Move from value_t macro to ArgMatches::value_of_t
- Typo in watch arg parsing

### 📚 Documentation

- Improve documentation across the board

### 🎨 Styling

- Fix lints
- Fix formatting
- Fix formatting

## [0.3.8] - 2021-12-30

### 🚀 Features

- *(node)* Output block hash as event data
- *(node)* Allow reading from arbitrary initial chain point (#10)
- Add common data aggregations to events (#13)
- Introduce 'filtering' stage (#14)
- Introduce 'fingerprint' filter (#16)
- Introduce 'selection' filter (#18)
- Add details to 'transaction' event (#24)

### 🐛 Bug Fixes

- *(node)* Handle non-map metadata structures (#12)

### 💼 Other

- Enable dependabot

### 📚 Documentation

- Move docker instructions to mdbook
- Improve 'watch' mode usage info
- Add fingerprint / selection filter docs

### 🎨 Styling

- Fix whitespaces
- Fix whitespaces (#17)

### ⚙️ Miscellaneous Tasks

- Fix branch filter for validate workflow
- Fix fmt / clippy warnings
- Fix fmt / clippy warnings
- Fix clippy / fmt warnings
- Add 'testdrive' workflow
- Update README feature check list

## [0.3.7] - 2021-12-23

### 🚀 Features

- *(elastic)* Improve Elasticsearch sink implementation (#8)

### 🐛 Bug Fixes

- Allow env override of nested configs

### ⚙️ Miscellaneous Tasks

- Fix fmt and clippy warnings

## [0.3.6] - 2021-12-22

### 🚀 Features

- Compute slot timestamp (#6)

### ⚙️ Miscellaneous Tasks

- Improve docker build speed (#7)
- Tidy up CI workflows

## [0.3.5-docker1] - 2021-12-19

### 🐛 Bug Fixes

- Include all features in docker build

### 📚 Documentation

- Link to Kafka's original site (#4)

## [0.3.5] - 2021-12-18

### 🚀 Features

- Include event type tag in serde output
- *(elastic)* Add Elasticsearch MVP Sink (#5)

### 🐛 Bug Fixes

- Valid entrypoint on Dockerfile
- Use correct cfg feature syntax
- Add all features in the binary releases

### 📚 Documentation

- Add draft documentation in mdbook format
- Point readme to mdbook
- Fill data dictionary and installation info
- Add context field to data dictionary
- Improve binary install example

## [0.3.4] - 2021-12-15

### 🐛 Bug Fixes

- *(n2c)* Remove v10 version constraint

## [0.3.3] - 2021-12-15

### 🐛 Bug Fixes

- Use correct cursor on chainsync roll forward #2

## [0.3.2] - 2021-12-13

### 🚜 Refactor

- Migrate to new version of pallas-alonzo

### 📚 Documentation

- Add cardano to kafka example

## [0.3.1-docker3] - 2021-12-12

### ⚙️ Miscellaneous Tasks

- Remove extra container platforms to speed up workflow

## [0.3.1-docker2] - 2021-12-12

### ⚙️ Miscellaneous Tasks

- Add docker build to release workflow

## [0.3.1] - 2021-12-10

### 🚀 Features

- Show rollback control event
- Represent addresses in bech32 format

### 🐛 Bug Fixes

- Remove local path dependencies

### ⚙️ Miscellaneous Tasks

- *(terminal)* Tidy up console output
- Remove unwraps and do a graceful exit
- Apply lint suggestions / formatting
- Bump version patch

## [0.3.0] - 2021-12-10

### 🚀 Features

- *(sources)* Implement node-to-node soure using chainsync + blockfetch
- *(terminal)* Improve output style
- Add certificate data mappings

### 📚 Documentation

- Add cli instructions

### 🎨 Styling

- Fix lint warnings

### ⚙️ Miscellaneous Tasks

- Explain extra dep on cargo
- Improve readme
- Improve source folder structure
- Bump version minor

## [0.2.0] - 2021-12-08

### 🚀 Features

- *(daemon)* Implement basic daemon cli subcommand
- *(kafka)* Add basic kafka sink implementation
- Compute and output tx hashes

### ⚙️ Miscellaneous Tasks

- Remove makefile
- Add use-cases to readme
- Improve args in log subcommand
- *(framework)* Define bootstrap traits for components
- Fix lint issues
- Fix lint issues
- Fix license year typo
- Add missing info to cargo metadata
- Bump version for release
- Fix missing openssl in release workflow
- Apply cargo fmt
- Fix missing openssl in release workflow

## [0.1.0] - 2021-12-05

### ⚙️ Miscellaneous Tasks

- Add github workflows
- Fix lint issues
- Add code of conduct
- Tidy up arg parsing in oura bin
- Fix lint issues

<!-- generated by git-cliff -->
