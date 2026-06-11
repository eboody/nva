# Event Types and Response Codes (Reference)

Source: https://support.gingrapp.com/hc/en-us/articles/25660010986125-Event-Types-and-Response-Codes-Reference
Section ID: 25380163240589

Event Types
 Once webhooks are activated in your application, a HTTP(s) request will be sent to the URL configured above when one of the following event occurs with relevant data in JSON format.

 Technical Event Name 

 Human Readable Event Name 

 Event Description 

 check_in 

 Reservation checked in

 This event is triggered when a reservation is checked in.

 check_out 

 Reservation checked out

 This event is triggered when a reservation is checked out

 checking_in 

 Reservation checking in

 This event is triggered when a user displays intent to check a reservation in. This can happen when a pet parent texts HERE to Gingr, or when an employee clicks on the check-in button from the dashboard.

 checking_out 

 Reservation checking out

 This event is triggered when a reservation is added to the shopping cart.

 email_sent 

 Email sent

 This event is triggered when a system generated email is sent from Gingr.

 owner_created 

 Owner created

 This event is triggered when a new owner record is created, either from the customer portal or the app by an employee.

 owner_edited 

 Owner edited

 This event is triggered when an existing owner record is updated, either from the customer portal or the app by an employee.

 animal_created 

 Animal created

 This event is triggered when a new animal record is created, either from the customer portal or the app by an employee.

 animal_edited 

 Animal edited

 This event is triggered when an existing animal record is updated, either from the customer portal or the app by an employee.

 incident_created 

 Incident created

 This event is triggered when a new incident is created for an animal by an employee in the app.

 incident_edited 

 Incident edited

 This event is triggered when an existing incident is updated for an animal by an employee in the app.

 lead_created 

 Lead created

 This event is triggered when a lead form is filled out by a pet parent either using the Gingr embeddable lead capture form or from the Gingr customer portal.

 HMAC Signature Verification
 Each request we send will contain a  signature  property. This is the output of a SHA256 HMAC function, where the message is "$webhook_type$entity_id$entity_type" (3 variables, concatenated together) and the key is the key set by you above. We recommend verifying the signature when you receive a request to verify that the request came from Gingr and was not forged by an imposter.
  
 Response Codes
 When you receive a webhook, you can let Gingr know that you were able to process the hook successfully. Use the following response codes:

 Response Code 

 Description 

 Helpful Hint 

 200 

 Webhook was successfully received and processed. 

 Gingr will not re-attempt delivery.

 403 

 Your application does not want this event. 

 Gingr will not re-attempt delivery.

 Any other response code 

 Some sort of error happened.  

 Gingr will retry 10 times, with increasing timeout durations between each subsequent attempt. 

   Notice:  Our Support Teams are not equipped to handle technical questions related to Webhook configuration beyond what's covered in our Knowledge Base articles regarding Webhooks implementation. We recommend seeking assistance from a third-party developer who can investigate and guide you through the process further should you need additional guidance.  
  
 Related Resources

 API & Integrations  Feature Overview

 API Topic Outline

 Webhooks Topic Outline

 Activate Webhooks How-To

 Event Types Data Structure Examples  Reference
