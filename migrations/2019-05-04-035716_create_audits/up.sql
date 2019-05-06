CREATE TABLE audits(
  addr VARCHAR NOT NULL,
  state VARCHAR NOT NULL,
  version VARCHAR NOT NULL,
  ts BIGINT,
  PRIMARY KEY (addr, ts)
)
