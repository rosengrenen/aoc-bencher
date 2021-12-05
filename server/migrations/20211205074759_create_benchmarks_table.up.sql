CREATE TABLE benchmarks (
  id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
  username VARCHAR NOT NULL,
  year BIGINT NOT NULL,
  day BIGINT NOT NULL CHECK (day >= 0 AND day <= 25),
  p1 BIGINT,
  p1_deviation BIGINT,
  p2 BIGINT,
  p2_deviation BIGINT,
  parse BIGINT,
  parse_deviation BIGINT,
  last_run TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  timeout BOOLEAN NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(), 
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  UNIQUE (username, year, day)
);

SELECT manage_updated_at('benchmarks');
