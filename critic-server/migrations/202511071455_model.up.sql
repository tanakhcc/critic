CREATE TYPE MODELTYPE AS ENUM('Recognition', 'Segmentation');

--- Table holding model metadata
CREATE TABLE model (
	id BIGINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
	--- the name of this model (NOTE: this is literally the file name and cannot be changed without renaming the file)
	name TEXT NOT NULL,
	--- is this model for recognition or segmentation
	model_type MODELTYPE NOT NULL,
	--- retrain every n days - can be null to not retrain at all
	--- 0 also does not retrain at all
	retrain_every_days INT,
	--- keep n version - can be null to keep all
	retrain_keep_versions INT
);
