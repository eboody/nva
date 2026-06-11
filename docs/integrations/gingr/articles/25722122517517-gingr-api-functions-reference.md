# Gingr API Functions (Reference)

Source: https://support.gingrapp.com/hc/en-us/articles/25722122517517-Gingr-API-Functions-Reference
Section ID: 25380109776397

Introduction
 Gingr offers a publicly available API (Application Programming Interface) to allow you to extract data and build additional functionality around our core product. 
 If you need data pulled from Gingr in a way that the core product will not support, then you can consult and hire a 3rd-party developer to help determine if and how  Gingr's API will work for you.
 Our support and development teams are not able to provide guidance for implementing the following instructions.  
  
 Use Gingr's API
 HTTP POST https://{your_app}.gingrapp.com/api/v1/reservations 

 retrieve a list of reservations within a given date range, or all currently checked in reservations

 required parameters key: 'user_specific_api_key'

 Required body:  
 checked_in: true (checked-in pets only) OR false (all reservations that match start/end dates)

 start_date: YYYY-MM-DD
 Maximum range is 30 days. Required only if checked_in = false, disregarded if checked_in = true 

 end_date: YYYY-MM-DD 
 Maximum range is 30 days. Required only if checked_in = false, disregarded if checked_in = true 

 Optional body: location_id: 1 (the location ID of a location in your app)

 Example cURL call

 HTML

 curl "https://{your_subdomain_here}.gingrapp.com/api/v1/reservations" \
-H 'Content-Type: application/x-www-form-urlencoded; charset=utf-8' \
--data-urlencode "start_date=YYYY-MM-DD" \
--data-urlencode "end_date=YYYY-MM-DD" \
--data-urlencode "checked_in=true" \
--data-urlencode "key={your_api_key_here}" 

 HTTP GET https://{your_app}.gingrapp.com/api/v1/reservation_widget_data 

 retrieve a summary of reservations for a given date, including # of check ins, outs and overnights.

 This is the API that powers the widget in the top-center of your Gingr Dashboard. 

 required parameters 
 key: 'user_specific_api_key',

 timestamp: a date in YYYY-MM-DD format

 HTTP POST https://{your_app}.gingrapp.com/api/v1/reservations_by_animal 

 retrieve a list of reservations for a given animal

 required parameters 
 key: - user's API Key

 id: 23 - an animal's ID

 optional parameters 
 restrict_to: choose "pending_requests", "currently_checked_in", "future", "past", "wait_listed"

 params: an array of 
 fromDate - ISO 8601 date format

 toDate - ISO 8601 date format

 reservationTypeIds - an array of reservation type IDs

 animalIds - an array of animal IDs

 cancelledOnly - boolean

 confirmedOnly - boolean

 completedOnly - boolean

 limit - number, only return these number of records

 Reference:   Reservation data for this call is only pulled for the location the User's API is currently logged into. If a pet has reservations for another location the user isn't currently operating in, that reservation info will not show up via API. 
  
 HTTP POST https://{your_app}.gingrapp.com/api/v1/reservations_by_owner 

 retrieve a list of reservations for a given owner

 required parameters 
 key: - user's API Key

 id: 23 - an owner's ID

 optional parameters 
 restrict_to: choose "pending_requests", "currently_checked_in", "future", "past", "wait_listed"

 params: an array of 
 fromDate - ISO 8601 date format

 toDate - ISO 8601 date format

 reservationTypeIds - an array of reservation type IDs

 animalIds - an array of animal IDs

 cancelledOnly - boolean

 confirmedOnly - boolean

 completedOnly - boolean

 limit - number, only return these number of records

 Reference:   Reservation data for this call is only pulled for the location the User's API is currently logged into. If a pet has reservations for another location the user isn't currently operating in, that reservation info will not show up via API. 

 HTTP GET https://{your_app}.gingrapp.com/api/v1/reservation_types 

 retrieve a list of reservation types

 required parameters key: 'user_specific_api_key',

 Optional Parameters 
 id (reservation type id)

 active_only true/false

 HTTP GET https://{your_app}.gingrapp.com/api/v1/get_services_by_type 

 retrieve a list of allowable additional services for a given reservation type

 Required Parameters 
 key (same as other calls)

 type_id (reservation type id, integer)

 Optional parameters location_id (location id, integer, probably 1)

 HTTP POST https://{your_app}.gingrapp.com/api/v1/authorize_owner 

 authorize that an owner account exists and a provided password matches what we have on file

 Required Parameters 
 email (customer email address)

 password (customers password in gingr)

 key (same as other calls)

 HTTP GET https://{your_app}.gingrapp.com/api/v1/report_card_files 

 retrieve a list of recently uploaded report card files

 required fields key (same as other calls)

 optional fields 
 number days - integer - (today - X days)

 limit - integer

 location_id

 HTTP POST https://{your_app}.gingrapp.com/api/v1/new_modified_owners 

 customers that were created or modified within the given date range

 required parameters 
 key: 'user_specific_api_key'

 start_date: 'YYYY-MM-DD',

 end_date: 'YYYY-MM-DD'

 optional parameters location_id --  if set, filters down to owners of the specified home location

 HTTP POST https://{your_app}.gingrapp.com/api/v1/recently_cancelled_reservations 

 reservations that were cancelled within a given date range

 required parameters 
 key: 'user_specific_api_key'

 start_date: 'YYYY-MM-DD'

 end_date: 'YYYY-MM-DD'

 optional parameters location_id --  if set, filters down to owners of the specified home location

 HTTP GET https://{your_app}.gingrapp.com/forms/get_form 

 returns the form's data structure for a type of record

 required parameters form -- either "owner_form" or "animal_form"

 HTTP GET https://{your_app}.gingrapp.com/api/v1/owner 

 Retrieve information about a specific owner record

 required parameters 
 key

 id - A gingr owner id

 optional parameters 
 id (owner ID)

 animal_id (animal ID)

 reservation_id (reservation_id)

 phone (cell phone number)

 email

 HTTP GET https://{your_app}.gingrapp.com/api/v1/get_locations 

 retrieve a list of locations for this app

 required parameters key

 HTTP GET https://{your_app}.gingrapp.com/api/v1/get_species 

 retrieve a list of species for this app

 required parameters key

 HTTP GET https://{your_app}.gingrapp.com/api/v1/get_breeds 
 
 retrieve a list of breeds for this app

 required parameters key

 HTTP GET https://{your_app}.gingrapp.com/api/v1/get_vets ?vetFlag=true 
 
 retrieve a list of vet names for this app

 required parameters key

 optional parameters vetFlag=true to return all vet information

 HTTP GET https://{your_app}.gingrapp.com/api/v1/get_temperaments 
 
 retrieve a list of temperaments for this app

 required parameters key

 HTTP GET https://{your_app}.gingrapp.com/api/v1/get_immunization_types 

 retrieve a list of immunizations for a given species

 required parameters 
 key

 species_id

 HTTP GET https://{your_app}.gingrapp.com/api/v1/get_animal_immunizations 

 retrieve a list of immunization records for a given animal

 required parameters 
 key

 animal_id

 HTTP GET https://{your_app}.gingrapp.com/api/v1/get_all_retail_items 

 retrieve a list of all retail items for sale

 required parameters key

 HTTP GET https://{your_app}.gingrapp.com/api/v1/existing_reservation_estimate 
 
 retrieve an estimated cost for a future reservation

 required parameters 
 key

 id (for a reservation)

 HTTP GET https://{your_app}.gingrapp.com/api/v1/list_transactions 
 
 Note: this API endpoint will only return POS Transactions before August 1, 2019.

 retrieve a list of transactions

 required parameters 
 key

 from_date: 'YYYY-MM-DD'

 to_date: 'YYYY-MM-DD'

 HTTP GET https://{your_app}.gingrapp.com/api/v1/list_invoices 

 Note: this API endpoint will only return Invoices created on/after August 1, 2019.

 retrieve a list of transactions

 required parameters key

 optional parameters 
 per_page -- integer; number of results to return (this + `page` param below enables pagination, requires `page` if used). defaults to null

 page -- integer; selects the result number to begin the page with (requires `per_page` if used). defaults to null.

 complete -- boolean (true or false); when set to true, will only return Invoices. When false, will only return Estimates. defaults to false.

 closed_only -- boolean (true or false); when set to true, will only return closed invoices. when set to false, will return open & closed invoices. defaults to false.

 from_date: 'YYYY-MM-DD'

 to_date: 'YYYY-MM-DD'

 Reference:   `per_page` and `page` must be used in conjunction. `page` will indicate the result number to begin the page, and should be incremented in accordance with `per_page`. For example, when `per_page=10`, the second page of results would begin with `page=11` and the third with `page=21`. 

 HTTP POST https://{your_app}.gingrapp.com/api/v1/transaction 

 retrieve a transaction and payment details

 required parameters 
 key

 id -- POS Transaction ID

 HTTP GET https://{your_app}.gingrapp.com/api/v1/timeclock_report 

 retrieve a list of timeclock records

 required parameters 
 key

 start_date: 'YYYY-MM-DD'

 end_date: 'YYYY-MM-DD'

 location_id -- integer

 optional parameters 
 include_deleted -- boolean

 include_clocked_in -- boolean

 user_ids -- array of user IDs

 HTTP GET https://{your_app}.gingrapp.com/api/v1/owners 

 retrieve a list of owners

 required parameters key

 optional parameters params: a key-value array of where clauses for the query

 Example cURL call that returns owners with a Zip code of 80302

 curl "https://{your-subdomain-here}.gingrapp.com/api/v1/owners" \
     -H 'Content-Type: application/x-www-form-urlencoded; charset=utf-8' \
     --data-urlencode "key={your-key-here}" \
     --data-urlencode "params[zip]=80302" 

 HTTP GET https://{your_app}.gingrapp.com/api/v1/animals 

 retrieve a list of animals

 required parameters key

 optional parameters params: a key-value array of where clauses for the query

 Example cURL call that returns animals with a birthday in November (using MySQL functions)

 HTML

 curl "https://{your_app}.gingrapp.com/api/v1/animals" \
 -H 'Content-Type: application/x-www-form-urlencoded; charset=utf-8' \
 --data-urlencode "params[month(from_unixtime(birthday))]=11" \
 --data-urlencode "key={user_specific_api_key}" 

 HTTP GET https://{your_app}.gingrapp.com/api/v1/get_subscription 
 
 retrieve a single subscription by its ID

 required parameters 
 key

 id -- ID of a subscription

 HTTP GET https://{your_app}.gingrapp.com/api/v1/get_subscriptions 
 
 retrieve a list of subscriptions

 required parameters key

 optional parameters 
 include_deleted -- true/false, whether canceled/deleted subscriptions should be returned in the response

 bill_day_of_month -- retrieve subscriptions set to renew on a specific day of the month

 owner_id -- an owner id, if set the response will be limited to subscriptions of that owner

 limit -- how many results to return in this response

 offset -- can be used in conjunction with limit to paginate the response

 location_id -- if set, response will be limited to a specific location

 package_id -- the ID of a particular package

 HTTP GET https://{your_app}.gingrapp.com/api/v1/custom_field_search 

 retrieve custom field information for an owner/animal

 parameters: 
 key: your api key

 form_id: id of the form you wish to search (owner_form = 1 animal_form = 2)

 field_name: technical name of field you wish to search

 search: the search term you are looking for on that field_name and form_id

 Example URL:
https://.gingrapp.com/api/v1/custom_field_search?key={{your_api_key}}&form_id=1&field_name=[your custom Field Name on the form]&search=[the search term you are needing to find] 
 results:
{
 "success": true,
 "error": false,
 "data": [{
 "system_id": "1694",
 "first_name": "Alexandra",
 "last_name": "Smith",
 "home_phone": "(555) 555-5555",
 "cell_phone": "(343) 444-4444",
 "emergency_contact_name": "Melissa Salminen",
 "emergency_contact_phone": "(222) 222-2222",
 "notes": "<b>Authorized Persons<\/b> Melissa Salminen<br>",
 "home_location": "1",
 "password": "adsfadsfasdfasfadsf",
 "stripe_default_card": null,
 "payment_processor_id": "1",
 "allow_online_login": "1",
 "opt_out_email": "0",
 "opt_out_sms": "0",
 "opt_out_marketing_email": "0",
 "opt_out_marketing_sms": "1",
 "opt_out_photo_sharing": "0",
 "opt_out_reminder_email": "0",
 "opt_out_reminder_sms": "0",
 "barcode": null,
 "submission_id": "10904",
 .....
 },
 ....
 ]
} 

 HTTP GET https://{your_app}.gingrapp.com/api/v1/back_of_house (AKA Digital Whiteboard) 

 retrieve data used to power Gingr's digital whiteboard.

 parameters: 
 key: your api key

 location_id (required)

 type_ids (required, an array of reservation_type_ids)

 mins_future (optional, if set, restricts to the next/last X minutes)

 full_day (optional, if set, mins_future field is ignored and system will include all reservations checking in/out today)

 Example request

 curl 'https://{subdomain}.gingrapp.com/api/v1/back_of_house?key={my_key}&location_id=1&full_day=true' 

  Example response

 {
 "success": true,
 "error": false,
 "data": {
 "checking_in": [
 {
 "id": "953",
 "owner_id": "94",
 "animal_id": "115",
 "o_last": "Holbrook",
 "a_first": "Maggie",
 "type_id": "1",
 "type": "Daycare | Full Day ",
 "check_in_stamp": null,
 "check_out_stamp": null,
 "start_date": "1559696400",
 "end_date": "1559703600",
 "run_name": null,
 "area_name": null,
 "belonging_count": "0",
 "belonging_area": null,
 "status_id": 2,
 "status_string": "Checking In Soon",
 "event_time": 1559696400
 }, 
 ...
 ],
 "checking_out": [
 {
 "id": "646",
 "owner_id": "8",
 "animal_id": "12",
 "o_last": "Douris",
 "a_first": "Sugar",
 "type_id": "5",
 "type": "Grooming Services",
 "check_in_stamp": "1559657671",
 "check_out_stamp": null,
 "start_date": "1559656800",
 "end_date": "1559664000",
 "run_name": null,
 "area_name": null,
 "belonging_count": "0",
 "belonging_area": null,
 "status_id": 2,
 "status_string": "Checking Out Soon",
 "event_time": 1559664000
 }, 
 ...
 ]
 }
} 

 HTTP GET https://{your_app}.gingrapp.com/api/v1/quick_checkin 

 checks in pet(s) for an existing reservation, if one does not exist it will create one and check it in.

 required parameters key

 optional parameters 
 animal_id

 owner_id

 type_id -- if set, system will use this reservation type id for new reservations. if not set, it will default to your system's quick type

 HTTP POST https://{your_app}.gingrapp.com/api/v1/receive_call 

 Notify Gingr of an incoming phone call. This will trigger an in-app alert in Gingr as well as record the call.

 required parameters 
 key

 From (phone number making the call)

 Called (phone number receiving the call)CallStatus (one of: initiated, ringing, answered, completed, in-progress, no-answer)

 CallSid (unique identifier for the call in the source system)

 optional parameters CallDuration (length of call in seconds)

 HTTP GET  https://{your_app}.gingrapp.com/api/v1/get_feeding_info 

 Retrieve an animal's feeding information

 required parameters 
 key

 animal_id

 HTTP GET  https://{your_app}.gingrapp.com/api/v1/get_medication_info 

 Retrieve an animal's medication information

 required parameters 
 key

 animal_id

 Related Resources

 API Topic Outline

 Get Started with API Topic Outline

 Access API Keys How-To

 Set Up Address Lookup and Owner Map in Google Maps How-To
