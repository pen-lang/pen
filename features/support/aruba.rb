# frozen_string_literal: true

require 'aruba/cucumber'

Aruba.configure do |config|
  config.exit_timeout = 1200
  config.home_directory = ENV['HOME']
end
