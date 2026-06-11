# Listen for Custom JavaScript Events (Reference)

Source: https://support.gingrapp.com/hc/en-us/articles/25846282748045-Listen-for-Custom-JavaScript-Events-Reference
Section ID: 25380161804813

Introduction

 You'll want to add your code that listens for custom JavaScript events in the  Customer app JS or Customer app footer  field on the  Custom Configurations  page. Below is an example listener function for the  reservation_created  event. 

 Example:

 Js

 document . addEventListener ( 'reservation_created' , function ( e ) { 
 // add conversion code here 
 // e.detail.reservation_ids is an object which contains the animal ID & reservation IDs of what was just created 
 } , false ) ; 

 Js

 document . addEventListener ( 'owner_created' , function ( e ) { 
 // add conversion code here 
 // e.detail is an object which contains the user supplied data that was submitted on the form. 
 } , false ) ; 

 Related Resources

 Custom CSS & JavaScript   Topic Outline

 Add Custom CSS & JavaScript How-To

 JavaScript Events Emitted Reference

 Google Tag Manager (Adwords/Analytics) in JavaScript Reference

 Facebook Pixel in JavaScript Events Reference
