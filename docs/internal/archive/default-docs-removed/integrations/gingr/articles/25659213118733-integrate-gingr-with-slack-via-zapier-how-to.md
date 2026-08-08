# Integrate Gingr with Slack via Zapier (How-To)

Source: https://support.gingrapp.com/hc/en-us/articles/25659213118733-Integrate-Gingr-with-Slack-via-Zapier-How-To
Section ID: 25380121365645

Introduction

 Using a combination of Gingr webhooks and Zapier , you can now push events that occur in Gingr to  Slack in real-time.

 Integrate Gingr with Slack via Zapier

 Here are the steps. It looks like a lot, but it can be completed in under 3 minutes!

 Visit Zapier's site and create an account (or log in if you have one already).

 Select the Make a Zap  button.

 Choose the Webhooks By Zapier  trigger type.

 Select Catch Hook .

 Copy the URL provided to you by Zapier.

 Visit your Gingr app.

 Navigate to Left Navigation Admin » Custom Configurations .

 Locate the Webhook URL field and paste the URL provided to you by Slack.

 Enter a random string of characters into the Webhook Signature Key field.

 Save your changes.

 Return to Zapier. Continue to the next step.

 In Gingr, perform an action that would trigger a webhook from Gingr. One example would be to submit a lead capture form, or check out a pet.

 Back in Zapier, the screen should've updated with webhooks it received from Gingr. Select the relevant one.

 Select Add a Step .

 Select Filter . Filter by Webhook Type .

 Select Add a Step .

 Select Action .

 Search for and select Slack .

 Select Send Channel Message .

 Authenticate with Slack, if necessary.

 Select a Slack channel to post to.

 In the message field, enter the message you want to post to Slack. You can use fields that Gingr passed to Zapier in the webhook to customize the message.

 Select Continue .

 Select Test .

 Check Slack to see your message!

 Related Resources

 API &   Integrations   Feature Overview

 Integrations Topic Outline

 Integrate Gingr with Facebook  How-To
