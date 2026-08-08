# Google Tag Manager (Adwords/Analytics) in JavaScript Events (Reference)

Source: https://support.gingrapp.com/hc/en-us/articles/25846389005325-Google-Tag-Manager-Adwords-Analytics-in-JavaScript-Events-Reference
Section ID: 25380161804813

Introduction

 Gingr's Customer Portal 2.0 allows you to embed Google Tag Manager and Facebook Pixel into the customer-facing application. This feature is not available for the customer portal native apps . This functionality is offered by Gingr, but requires the help of a 3rd party web developer hired by your business. Our support and development teams are not able to provide guidance for implementing the following instructions.

 IMPORTANT!  The Customer App CSS Field and Customer App JS field  are non-functional in Customer Portal 2.0.

 JavaScript Events

 JavaScript events can be used to trigger a conversion in Google AdWords, Facebook Pixel etc. There are three options for Google Tag Manager events:

 reservation_created -  This event is emitted when a reservation or appointment request has been made

 owner_created -  This event is emitted when a customer registers a new account

 lead_created - This event is emitted when a customer submits a lead form

 Requirements

 An existing Google Tag Manager (GTM) and Google Analytics 4 (GA4) account

 An existing GTM container and knowledge of the GTM ID

 Awareness of your measurement ID (typically prefixed with “G-” and found in Google Analytics > Admin > Data Streams > “Measurement ID“)

 Identification of the specific event(s) you intend to track in GA4

 Step 1: Create Google Analytics GA4 Configuration GTM Tag for Initializing on All Pages 

 Go to your Google Tag Manager container, click on Tags > New, and name your tag (i.e. "Google Tag G-Measurement_id").

 Click on Tag Configuration > Choose tag type > Google Tag.

 Enter your GA4 Measurement ID in the "Measurement ID" field (typically prefixed with “G-” and found in your Google Analytics account)

 Under Triggering, click Add Trigger > Choose a trigger > Initialization - All Pages.

 Step 2: Create a Custom HTML Tag

 Go to your Google Tag Manager container, click on Tags > New, and name your tag (i.e. "Custom HTML").

 Click on Tag Configuration > Choose tag type > Custom HTML.

 For Triggering, choose “All Pages” so this gets inserted when a page is viewed.

 The following is the HTML code to be used. Replace <measurement ID> with your measurement ID (typically prefixed with “G-” and found in your Google Analytics account):

 <!-- Global site tag (gtag.js) - Google Analytics -->
<script async src="https://www.googletagmanager.com/gtag/js?id=<measurement ID>"></script>
<script>
window.dataLayer = window.dataLayer || [];
function gtag(){
window.dataLayer.push(typeof arguments[0] == 'object' ? arguments[0] : arguments);
}
gtag('js', new Date());

 gtag('config', '<measurement ID>');
</script>

 Step 3: Generate a Custom Event Trigger 

 Prior to crafting the custom event tag, you must first create a custom trigger. This is necessary to instruct Google Tag Manager (GTM) to activate when a specific event occurs.

 Navigate to Triggers -> New, and opt for the 'Custom Event' configuration.

 Name the trigger “owner_created”, “reservation_created”, or “lead_created” depending on what GTM event you are tracking.

 Name the event “owner_created”, “reservation_created”, or “lead_created”.

 Set the trigger to fire on “All Custom Events”.

 Step 4: Generate an Event Tag 

 Navigate to the Tags section and click New, and name your tag (i.e. "owner_created").

 Click on Tag Configuration > Choose tag type > Google Analytics > Google Analytics: GA4 Event.

 Enter your GA4 Measurement ID in the "Measurement ID" field (typically prefixed with “G-” and found in your Google Analytics account)

 For the trigger, select the custom event trigger you generated earlier (i.e. “owner_created”)

 At this stage, assuming the tags outlined in this guide are the sole ones you've generated, your tags page should appear as follows: 

 Step 5: Gingr Admin Setup 

 Login to your Gingr app and navigate to Admin -> Advanced Settings -> Custom Configurations.

 Within the "Customer app footer" section, paste the following code snippet.

 Replace GTM-ID with your actual GTM ID (usually prefixed with GTM- ) and click outside of the field to ensure that the changes are saved.

 Replace the event listener with the applicable event you are listening for (reservation_created, owner_created, or lead_created).

 <!-- Google Tag Manager -->
(function(w,d,s,l,i){w[l]=w[l]||[];w[l].push({'gtm.start': 
new Date().getTime(),event:'gtm.js'});var f=d.getElementsByTagName(s)[0],
j=d.createElement(s),dl=l!='dataLayer'?'&l='+l:'';j.async=true;j.src= 
'https://www.googletagmanager.com/gtm.js?id='+i+dl;f.parentNode.insertBefore(j,f);
})(window,document,'script','dataLayer','GTM-ID');
<!-- End Google Tag Manager -->

 <script>

 document.addEventListener('owner_created', function(e) {
window.dataLayer.push({
event: 'owner_created',
ownerData: e.detail
});
});

 </script>

 Testing

 To test that this works, in GTM click “Preview” in the top banner on the page. This will take you to a page asking you for the domain to connect to. Put in your customer portal URL (usually https://<app>.portal.gingrapp.com ) and click “Connect”.

 Your customer portal will open in a new tab or window. Log in as a customer and click on “More” on the menu and then “Contact us”. Fill out the form and submit. Now go back to the Tag Manager tab (it’s icon will be flashing between an empty and blue tag icon) where you put in your customer portal URL , and you should see something like this on the left side:

 On the right side after you select the event you should see something like this:

 The important thing is to see the GA4 Event created earlier in the “Tags Fired” section.

 In your Google Analytics Realtime Report you should also see the user session and the event listed.

 Going Live

 Once satisfied go to GTM and click “Submit” then publish the container. Changes may take a few moments to take effect between the your app and Google.

 Related Resources

 Custom CSS & JavaScript   Topic Outline

 Add Custom CSS & JavaScript How-To

 JavaScript Events Emitted Reference

 Listen for Custom JavaScript Events Reference

 Facebook Pixel in JavaScript Events Reference
