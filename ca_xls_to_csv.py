import openpyxl
import csv
from pprint import pprint
from xl_utils import get_key_values_list
# Open excel file
FILENAME = "CA20220405_001636.xlsx"
CSV_FILENAME = "generated_csv.csv"
key_col_dict = {
	"date": "A",
	"libelle": "B",
	"debit": "C",
	"credit": "D",
}
workbook = openpyxl.load_workbook(filename=FILENAME, read_only=True,
                                  keep_vba=True)
accounting_objects = get_key_values_list(workbook, "Sheet0", key_col_dict, 
	                                     11, None)
	# well actually the first line (11) argument seems to be ignored as 1st line is merged cells... dammit
# process values
def get_values(cells_dict):
	return {key: dict_value.value 
	        for key, dict_value in cells_dict.items()}

def is_dummy_entry(accounting_object):
	""" Damn you, CA"""
	return \
		"Téléchargement" in str(accounting_object["date"].value) \
		or accounting_object["date"].value is None \
		or accounting_object["date"].value == "Date" \
		or (accounting_object["credit"].value is None and accounting_object["debit"].value is None) 

def process_values(accounting_object):
	accounting_object_mod = get_values(accounting_object)
	if accounting_object_mod["libelle"] is None:
		accounting_object_mod["libelle"] = ""
	else:
		# remove \n
		accounting_object_mod["libelle"] = accounting_object_mod["libelle"]\
			.replace("\n", " ").strip()
		# Substitute multiple whitespace with single whitespace
		accounting_object_mod["libelle"] = ' '.join(accounting_object_mod["libelle"].split())

	print(accounting_object_mod)
	accounting_object_mod["date"] = accounting_object_mod["date"].strftime("%m/%d/%Y")
	accounting_object_mod["debit_credit"] = \
		-1. *accounting_object_mod["debit"] if accounting_object_mod["credit"] is None \
		else accounting_object_mod["credit"]
	return accounting_object_mod

accounting_objects = [process_values(accounting_object) 
                      for accounting_object in accounting_objects
                      if not is_dummy_entry(accounting_object)]
# change to target CSV format 
# dump CSV
pprint(accounting_objects)


with open(CSV_FILENAME, 'w', newline='') as csvfile:
    csv_writer = csv.writer(csvfile, delimiter=',',
                            quotechar='|', quoting=csv.QUOTE_MINIMAL)
    csv_writer.writerow([
        "date_operation",
        "date_debit_credit",
        "debit_credit",
        "libelle",
        "balance",
    ])
    for accounting_object in accounting_objects:
	    csv_writer.writerow([
	    	accounting_object["date"],
	    	accounting_object["date"],
	    	accounting_object["debit_credit"],
	    	accounting_object["libelle"],
	    	0, # the balance...
	    ])
