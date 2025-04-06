# User Guide

This guide provides instructions for using the Discord Bot Web Dashboard.

## Table of Contents

1. [Getting Started](#getting-started)
2. [Dashboard Overview](#dashboard-overview)
3. [Guild Management](#guild-management)
4. [Command Configuration](#command-configuration)
5. [Word Detection Rules](#word-detection-rules)
6. [Settings Management](#settings-management)
7. [Analytics](#analytics)
8. [Troubleshooting](#troubleshooting)

## Getting Started

### Accessing the Dashboard

1. Go to the dashboard URL provided by your administrator.
2. Click "Login with Discord" to authenticate.
3. Grant the required permissions when prompted by Discord.
4. You will be redirected to the dashboard home page.

### Requirements

- A modern web browser (Chrome, Firefox, Safari, Edge)
- Discord account with administrator permissions on at least one server
- The bot must be added to your Discord server

### Adding the Bot to Your Server

If the bot is not already in your server:

1. From the dashboard home page, click "Add to Server".
2. Select the server you want to add the bot to from the Discord popup.
3. Review and grant the requested permissions.
4. You will be redirected back to the dashboard.

## Dashboard Overview

The dashboard is organized into the following sections:

- **Home**: Overview of your servers and recent activity
- **Guild Management**: Server-specific settings and information
- **Command Configuration**: Enable/disable and configure bot commands
- **Word Detection**: Configure automated text moderation
- **Settings**: General bot settings for your server
- **Analytics**: Usage statistics and metrics

The left sidebar allows quick navigation between your servers and different sections.

## Guild Management

### Server Selection

1. Click on a server in the left sidebar to manage that specific server.
2. The server overview page shows basic information like member count, enabled commands, and active word rules.

### Server Information

The server information panel displays:

- Server name and icon
- Member count
- Bot status
- Command statistics
- Word detection statistics

## Command Configuration

### Enabling/Disabling Commands

1. Navigate to the Commands page for your server.
2. Toggle the switch next to a command to enable or disable it.
3. Changes take effect immediately.

### Configuring Command Settings

1. Click on a command to expand its settings.
2. Modify the available options.
3. Click "Save" to apply the changes.

### Bulk Management

1. Use the category dropdowns to filter commands by category.
2. Use the "Select All" and "Deselect All" buttons for bulk actions.
3. Click "Enable Selected" or "Disable Selected" to perform bulk actions.

## Word Detection Rules

Word detection rules allow the bot to automatically moderate messages based on patterns.

### Creating a New Rule

1. Navigate to the Word Detection page.
2. Click "Add New Rule".
3. Enter the pattern to detect (can be text or a regular expression).
4. Select the action to take when the pattern is detected:
   - Delete: Removes the message
   - Warn: Warns the user
   - Timeout: Times out the user
   - Kick: Kicks the user
   - Ban: Bans the user
5. Configure any action-specific parameters.
6. Click "Save Rule".

### Testing a Rule

1. On the Word Detection page, click "Test Rules".
2. Enter sample text in the test area.
3. Click "Test" to see which rules would be triggered.

### Editing or Deleting Rules

1. Find the rule in the list.
2. Click the edit icon to modify it.
3. Click the delete icon to remove it.
4. Confirm any deletion when prompted.

## Settings Management

### General Settings

1. Navigate to the Settings page.
2. Configure the following options:
   - Command Prefix: The character(s) that trigger bot commands
   - Log Channel: Where bot activity is logged
   - Moderation Settings: General moderation options

### Auto-Moderation Settings

1. In the Settings page, find the Auto-Moderation section.
2. Enable or disable automatic moderation features:
   - Link filtering
   - Discord invite filtering
   - Profanity filtering
3. Configure thresholds and exceptions.

### Welcome Message Settings

1. In the Settings page, find the Welcome Message section.
2. Enable or disable welcome messages.
3. Configure the welcome message channel.
4. Customize the welcome message text:
   - Use `{user}` to mention the new member
   - Use `{server}` to include the server name
5. Save your changes.

## Analytics

The Analytics dashboard provides insights into bot usage and performance.

### Viewing Statistics

1. Navigate to the Analytics page.
2. View summary statistics at the top:
   - Total commands used
   - Active users
   - Messages processed
   - Rules triggered

### Filtering Data

1. Use the server dropdown to filter by a specific server.
2. Use the time range dropdown to select a time period:
   - Last 7 days
   - Last 30 days
   - Last 90 days
   - Custom range
3. If selecting a custom range, specify the start and end dates.

### Understanding Charts

1. **Command Usage Chart**: Shows which commands are used most frequently.
2. **User Activity Chart**: Shows message volume and command usage by day of week or time period.

### Exporting Data

1. Hover over any chart to see specific data points.
2. Use your browser's screenshot functionality to capture charts for reports.

## Troubleshooting

### Common Issues

1. **Cannot see a server in the dashboard**
   - Ensure you have administrator permissions on the server
   - Verify the bot has been added to the server
   - Try logging out and back in

2. **Command not working in Discord**
   - Check if the command is enabled in the dashboard
   - Verify you're using the correct prefix
   - Ensure the bot has proper permissions in your server

3. **Changes not taking effect**
   - Some changes may take a few minutes to propagate
   - Ensure you clicked "Save" after making changes
   - Try refreshing the dashboard

### Getting Help

If you continue to experience issues:

1. Check the documentation for updated information
2. Contact the bot administrator
3. Join the support server (if available)

### Reporting Bugs

If you encounter a bug:

1. Note the steps to reproduce the issue
2. Take screenshots if applicable
3. Report the issue to the administrator
