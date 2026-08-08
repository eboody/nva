# Set Up Address Lookup (How-To)

Source: https://support.gingrapp.com/hc/en-us/articles/26785597375885-Set-Up-Address-Lookup-How-To
Section ID: 26779515142157

Set Up Address Lookup in Gingr

 Navigate to: https://cloud.google.com/maps-platform/#get-started 

 A box 'Enable Google Maps Platform' will populate and you will select 'Maps' > continue.

 Name your Project.

 Create your Billing Account and select Preferences.

 Create your New Project.

 Ensure that the Places API is enabled. To do this, navigate to APIs and Services » Library and search for Places API . Enable this API.

 On the API Key settings page you'll adjust these settings:

 Application Restrictions : HTTP referrers (websites)

 Accept requests from these HTTP referrers (websites) :

 Insert your Gingr app URL: " https://businessname.gingrapp.com/* ". Note that there is an * at the end of the URL. This is required to allow the API to communicate with the various pages in Gingr that may use Address Look-Up.

 Once your HTTP Referrers are properly set, copy your newly configured API Key

 Navigate to your Gingr app Left-hand Navigation: Admin » Google and enter your copy/pasted API key, and save.

 Related Resources

 Google Maps API Topic Outline

 Set Up Owner's Map How-To
