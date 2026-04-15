-- Migration: Add switch columns to sensor_data table
-- Date: 2025-04-15

-- Add digital_input_0 column (main hook switch)
ALTER TABLE sensor_data ADD COLUMN digital_input_0 BOOLEAN NOT NULL DEFAULT 0;

-- Add digital_input_1 column (auxiliary hook switch)
ALTER TABLE sensor_data ADD COLUMN digital_input_1 BOOLEAN NOT NULL DEFAULT 0;