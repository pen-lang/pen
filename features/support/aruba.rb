# frozen_string_literal: true

require 'aruba/cucumber'

Aruba.configure do |config|
  config.exit_timeout = 600
  config.home_directory = ENV['HOME']
end
