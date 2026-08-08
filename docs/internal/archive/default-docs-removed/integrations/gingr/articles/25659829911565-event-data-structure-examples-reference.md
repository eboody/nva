# Event Data Structure Examples (Reference)

Source: https://support.gingrapp.com/hc/en-us/articles/25659829911565-Event-Data-Structure-Examples-Reference
Section ID: 25380163240589

Reservation Events

 Reservation  events have the following structure. Please note that the  check_out event type will include invoice data for the checked out reservation that is not included in the example below.
  
 {
     "webhook_url": " https://borg.gingr.io/webhooks.php ",  //The URL Gingr is sending this request to 

     "webhook_type": "check_out",  //The event type that occurred 

     "entity_id": "76390",  //The ID of this webhook. I.e. a reservation ID 

     "entity_type": "reservation",  //The type of entity of this webhook. I.e. a reservation 

      "signature": "91a12089c1223b700b7afd4f86b90105285bf9d58984a7d2042fc483f3e7d4f5",  //The SHA256 HMAC of the {webhook_type} {entity_id} and {entity_type} fields concatenated together using the Signature Key set in System wide Settings 
 
     "entity_data": {  //The data associated with the above entity type and entity ID 

         "a_id": "17812",
         "animal_id": "17812",
         "animal_name": "Bella",
         "gender": "Female",
         "fixed": "1",
         "vip": "0",
         "medicines": "",
         "allergies": null,
         "image": null,
         "species_id": "1",
         "a_notes": null,
         "grooming_notes": "<p>3/14/16 - Did not need NT. Nails are already really short.&nbsp;</p>",
         "birthday": "1430452800",
         "weight": "1",
         "pricing_rules_apply": "1",
         "o_id": "11277",
         "o_first": "Joe",
         "o_last": "Smith",
         "address_1": "626 Apple St.",
         "address_2": null,
         "city": "Winston Salem",
         "state": "North Carolina",
         "zip": "27103",
         "email": "demo@gmail.com",
         "home_phone": "(303) 555-5555",
         "cell_phone": "(303) 555-5555",
         "emergency_contact_name": "",
         "emergency_contact_phone": "",
         "o_notes": null,
         "current_balance": "0.00",
         "default_payment_method_fk": null,
         "stripe_customer_id": null,
         "vet_id": "6375",
         "vet_name": "Mary Vista",
         "vet_phone": "(303) 555-5555",
         "barcode": null,
         "r_id": "76390",
         "start_date": "1471251600",
         "end_date": "1471309200",
         "r_notes": "Please give them extra love!",
         "confirmed_stamp": "1471207352",
         "cancel_stamp": null,
         "check_in_stamp": "1471262703",
         "check_out_stamp": "1471452858",
         "wait_list_stamp": null,
         "wait_list_accepted_stamp": null,
         "location_id": "3",
         "last_email_sent": "1471452858",
         "last_sms_sent": null,
         "type_id": "1",
         "self_made": null,
         "answer_1": null,
         "answer_2": null,
         "answer_3": null,
         "created_at": "1471205155",
         "base_rate": "27.00",
         "final_rate": "27.00",
         "units_of_time": "3",
         "created_by": "vanessa  williams",
         "breed_id": "46",
         "breed_name": "Golden Retriever",
         "feeding_method": "Feed alone",
         "feeding_type": "Own Dry Food",
         "feeding_notes": "Own dry- 2 cups in am, 2 cups in pm",
         "temperment_type": null,
         "default_payment_method": null,
         "payment_amount": null,
         "type": "Daycare: Full Day",
         "capacity": "100",
         "question_1": null,
         "question_2": null,
         "question_3": null,
         "charge_by_hour": "0",
         "location_name": "Happy Wags",
         "location_city": "Boulder",
         "cancellation_reason": null,
         "cancellation_reason_id": null,
         "cancelled_by_username": null,
         "services_string": null,
         "feeding_time": "AM,PM",
         "feeding_amount": "2,2",
         "start_date_iso": "2016-08-15T08:05:03-04:00",
         "end_date_iso": "2016-08-15T21:00:00-04:00"
     }
 }
  
 Email Events
 Email  events have the following structure:
  
 {
     "webhook_url": " https://borg.gingr.io/webhooks.php ",   //The URL Gingr is sending this request to 
     "webhook_type": "email_sent",  //The event type that occurred 
     "entity_id": 5917,   //The ID of this webhook. I.e. a reservation ID 
     "entity_type": "owner",  //The type of entity of this webhook. I.e. a reservation 
     "signature": "91a12089c1223b700b7afd4f86b90105285bf9d58984a7d2042fc483f3e7d4f5",  //The SHA256 HMAC of the {webhook_type} {entity_id} and {entity_type} fields concatenated together using the Signature Key set in System wide Settings 
 
     "entity_data": {  //The data associated with the above entity type and entity ID 
         "id": "5917",
         "first_name": "Mickey",
         "last_name": "Mouse",
         "address_1": "841 Lookout Rd",
         "address_2": null,
         "city": "Winston Salem",
         "state": "North Carolina",
         "zip": "27104",
         "email": "demo@gmail.com",
         "home_phone": "(303) 555-5555",
         "cell_phone": "(303) 555-5555",
         "emergency_contact_name": "",
         "emergency_contact_phone": "",
         "notes": null,
         "default_payment_method_fk": null,
         "home_location": "2",
         "opt_out_email": "0",
         "opt_out_sms": "0",
         "opt_out_marketing_email": "0",
         "opt_out_marketing_sms": "0",
         "opt_out_photo_sharing": "0",
         "opt_out_reminder_email": "0",
         "opt_out_reminder_sms": "0",
         "current_balance": "-2.56",
         "latitude": "35.23509",
         "longitude": "-75.1280061",
         "stripe_customer_id": null,
         "stripe_default_card": null,
         "payment_processor_id": "1",
         "key": "576bf720b3b6a",
         "barcode": null,
         "allow_online_login": "1",
         "created_at": "1361422800",
         "created_by": null,
         "kl_owner_id": "1002782-2",
         "form_token": "577fd1f2c5347"
     },
     "email_data": {  //The data for the email that was sent 
         "subject": "Test email subject",
         "value": "<p>Test email message</p><div style=\"line-height: 22.2222px;\"><br></div>"
     },
     "recipients": [  //The recipients of this email 
         {
             "name": "Mickey Mouse",
             "email": "demo@gmail.com"
         }
     ]
 } 
  
 Notice:  Our Support Teams are not equipped to handle technical questions related to Webhook configuration beyond what's covered in our Knowledge Base articles regarding Webhooks implementation. We recommend seeking assistance from a third-party developer who can investigate and guide you through the process further should you need additional guidance.  
 Related Resources

 API & Integrations  Feature Overview

 API Topic Outline

 Webhooks Topic Outline

 Activate Webhooks How-To

 Event Types and Response Codes Reference
