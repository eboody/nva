# Set Up Address Lookup and Owner Map in Google Maps API (How-To)

Source: https://support.gingrapp.com/hc/en-us/articles/25722435154573-Set-Up-Address-Lookup-and-Owner-Map-in-Google-Maps-API-How-To
Section ID: 25380109776397

Introduction

 This article will guide you through how to create and access your Google Maps API key and enter it into Gingr to enable features like Address Look-Up and Owner Map.

 Reference:   On July 16, 2018, Google implemented a new pay-as-you-go pricing plan that went into effect for Maps, Routes, and Places. This caused a sharp increase in cost for Gingr to include this offer for free. Because of this, we have removed the address look-up feature in Gingr as an included feature.

 However, we've created a setting for an API entry so that you can set-up your own Google Maps account that connects with Gingr for Address Look-Up. This article will show you how to access that API key from Google and enter that API key into Gingr. 

 Before You Begin

 You will need to set up and configure a Google Cloud Platform account to enable these features. Costs may apply. Please carefully understand these costs and your options by visiting Google's pricing page in this article: Platform Pricing & API Costs .

 If you have multiple locations in one app URL, you only need to implement these instructions one time. The setup will apply to all locations in your application.

 Video Tutorial

 Address Lookup

 Navigate to:  https://cloud.google.com/maps-platform/#get-started .

 A box 'Enable Google Maps Platform' will populate and you will select Maps > continue.

 Name your Project.

 Create your Billing Account and select Preferences.

 Create your New Project.

 Ensure that the  Places API  is enabled. To do this, navigate to  APIs and Services » Library and search for  Places API . Enable this API.

 On the API Key settings page you'll adjust these settings:

 A.)  Application Restrictions:  HTTP referrers (websites) 

 Accept requests from these HTTP referrers (websites): 

 B.)  Insert your Gingr app URL: " https:// businessname.gingrapp.com/* "

 Note that there is an * at the end of the URL. This is required to allow the API to communicate with the various pages in Gingr that may use Address Look-Up

 Once your HTTP Referrers are properly set, copy your newly configured API Key

 Navigate to your Gingr app  Left-hand Navigation: Admin » Google  and  Enter your copy/pasted API key, and save.

 Owner Map

 Navigate to:  https://cloud.google.com/maps-platform/#get-started   

 Reference:   If you are already using the Address Lookup feature, you can skip to step 6. 

 A box 'Enable Google Maps Platform' will populate and you will select 'Maps' > continue. 

 Name your Project. 

 Create your Billing Account and select Preferences. 

 Create your New Proj ect. 

 Ensure that  two API keys exist. You can create a new API key by going to  API's and Services » Credentials » Create Credentials » API Key 

 Important:   If you already have a maps API key for address lookup, you DO NOT need to create a new project (unless you want to bill the owner map and address lookup separately). You can instead create a  separate, unrestricted  API key for the Owner Map. 

 On the API Key settings page you'll adjust these settings to restrict one  of your API keys: ( Note: if you already have a restricted API key set up for the Address Lookup feature, you can move to the next step) 

 Application Restrictions:  HTTP referrers (websites) 

 Accept requests from these HTTP referrers (websites): 

 Insert your Gingr app URL: " https:// businessname.gingrapp.com/* "
 
 Note that there is an * at the end of the URL. This is required to allow the API to communicate with the various pages in Gingr that may call the Google API

 Ensure that your second API key is unrestricted (no application or API restrictions). 

 Ensure that the Geocoding  API  and  Maps Javascript API are enabled. To do this, navigate to  APIs and Services » Library and search for each of these APIs. Enable these APIs. 

 Navigate to your Gingr app  Left-hand Navigation: Admin » Google . Enter your copy/pasted API key and save. 

 Important:   To fully enable the owner map feature, you must have an HTTP referrer restricted  API key entered into the Google Maps API Key field in Gingr and an  unrestricted   API key entered into the  Google Maps  Secret API Key  field in Gingr. 

 Warning:   Do not put an unrestricted API key into the Google Maps API Key field. 

 You're all set! You can now view your owners by address on a map by going to  Left Navigation: Owners and Pets » View Map . 
 Important:   The first time that you load the Owner Map page, you'll notice that the page takes a bit longer to load than you might expect! This is normal and occurs only on the first time that you access this page. The system has to go through all the owners and ask Google for the latitude/longitude for each owner in your system. Gingr will do this on the backend for you over a period of a few days once you enter your API key. Depending on how many owners your app has, this process can take a few days for the map to fully populate! If you have a considerable amount of owners in your Gingr database (+5k), reach out to support for assistance! 

 Related Resources

 API Topic Outline

 Get Started with API Topic Outline

 Gingr API Functions Reference

 Access API Keys How-To
