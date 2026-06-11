# Access API Keys (How-To)

Source: https://support.gingrapp.com/hc/en-us/articles/25721874372109-Access-API-Keys-How-To
Section ID: 25380109776397

Introduction 

 Before you can start using our API, you must first have a key. All API requests require a key that is based on a user account. You can choose to use your own user account's API key or create a new user account for the purposes of accessing the API.  

 Before You Begin

 Requests are performed with HTTP over TLS (HTTPS) and will return a JSON object as a result. All endpoints are read-only.

 Important:   Never share your API keys with any party that should not be able to access your Gingr data. If you are using an employee's API key and they are no longer with the company, or if you think your API key has been exposed, you can delete and re-create a new key for a user account 

 Access a User's API Key

 To retrieve a user's key, go to the Left-hand Navigation: Reports & More » Users »  Edit User » API Keys . 

 The user account for making API requests must have the  Can Access API user permission from Left-hand Navigation: Reports and More Icon » Groups .  

 When making an API request, you will need to include the user's API key to make a successful request. 

 Reference:   You will see example requests reference the key parameter as "{my_key}". 

 Related Resources

 API Topic Outline

 Get Started with API Topic Outline

 Gingr API Functions Reference

 Set Up Address Lookup and Owner Map in Google Maps How-To
