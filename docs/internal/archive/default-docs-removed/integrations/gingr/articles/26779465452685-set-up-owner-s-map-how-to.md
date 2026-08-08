# Set Up Owner's Map (How-To)

Source: https://support.gingrapp.com/hc/en-us/articles/26779465452685-Set-Up-Owner-s-Map-How-To
Section ID: 26779515142157

Set Up Owner's Map

 Navigate to: https://cloud.google.com/maps-platform/#get-started 

 Reference:   If you are already using the Address Lookup feature, you can skip to step 6. 

 A box 'Enable Google Maps Platform' will populate and you will select Maps > Continue.

 Name your Project.

 Create your Billing Account and select Preferences.

 Create your New Project.

 Ensure that two API keys exist. You can create a new API key by going to API's and Services » Credentials » Create Credentials » API Key .

 Important:   If you already have a maps API key for address lookup, you DO NOT need to create a new project (unless you want to bill the owner map and address lookup separately). You can instead create a separate, unrestricted API key for the Owner Map. 

 On the API Key settings page you'll adjust these settings to restrict one of your API keys:
 Reference:   If you already have a restricted API key set up for the Address Lookup feature, you can move to the next step. 

 Application Restrictions : HTTP referrers (websites)

 Accept requests from these HTTP referrers (websites) :

 Insert your Gingr app URL: " https://businessname.gingrapp.com/* ". Note that there is an * at the end of the URL. This is required to allow the API to communicate with the various pages in Gingr that may call the Google API.

 Ensure that your second API key is unrestricted (no application or API restrictions).

 Ensure that the Geocoding API and Maps Javascript API are enabled. To do this, navigate to APIs and Services » Library and search for each of these APIs. Enable these APIs.

 Navigate to your Gingr app Left-hand Navigation: Admin » Google . Enter your copy/pasted API key and save.

 Important:   To fully enable the owner map feature, you must have an HTTP referrer restricted API key entered into the Google Maps API Key field in Gingr and an unrestricted API key entered into the Google Maps Secret API Key field in Gingr. 

 Warning:   Do not put an unrestricted API key into the Google Maps API Key field. 

 You're all set! You can now view your owners by address on a map by going to Left Navigation: Owners and Pets » View Map . 

 Important:   The first time that you load the Owner Map page, you'll notice that the page takes a bit longer to load than you might expect! This is normal and occurs only on the first time that you access this page. The system has to go through all the owners and ask Google for the latitude/longitude for each owner in your system. Gingr will do this on the backend for you over a period of a few days once you enter your API key. Depending on how many owners your app has, this process can take a few days for the map to fully populate! If you have a considerable amount of owners in your Gingr database (+5k), reach out to support for assistance! 

 Related Resources

 Google Maps API Topic Outline

 Set Up Address Lookup How-To
