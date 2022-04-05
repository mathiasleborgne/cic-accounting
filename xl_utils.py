#!/usr/bin/env python
# -*- coding: utf-8 -*-

""" Collection of Excel parsing utilities, mostly openpyxl wrappers
    Most important functions:
        - open_xls_input
        - get_key_values_list
"""
from __future__ import unicode_literals

# Used to open to corrupt excel file
import io
import os
import re
from os.path import join

import openpyxl
# useful functions -------------------------------------------------------------
from openpyxl.cell.read_only import EmptyCell
from openpyxl.utils import column_index_from_string


def get_key_values_list(work_book, sheet_name, key_column_dict, first_index, not_none_key):
    """ Get Cell dictionaries from an excel file, given a description of columns
        returns [{key:cell, ...}, ...]
            eg: [{
                "data_name": Cell("speed"),
                "type": Cell("float"),
            }, {
                "data_name": Cell("latitude"),
                "type": Cell("float"),
            }, ... ]
        with args:
            key_column_dict: {key: column_ID, ...}
                eg: {
                    "data_name": "A",
                    "type": "B",
                    "description": "C",
                }
            not_none_key: name of key in key_column_dict that shouldn't be an empty Cell
                          (else it's filtered out), use None to avoid this behaviour
            first_index: index (excel index) of first values line
                         (eg if line1: title and lines2-15: values,
                         then first_index = 2)
    """
    work_sheet = work_book[sheet_name]
    key_column_ID_int_dict = {
        key: openpyxl.utils.column_index_from_string(column_ID) - 1
        for key, column_ID in key_column_dict.items()
    }
    max_column = max(key_column_ID_int_dict.values()) + 1
    key_values_list = []  # can't do it with direct column selection in read-only...
    # work_sheet.reset_dimensions()
    work_sheet.calculate_dimension(force=True)  # Fix max_rows wrong size from excel file
    for row_cells in work_sheet.rows:
        try:
            if not_none_key is not None:
                index_not_none_key = key_column_ID_int_dict[not_none_key]
                # todo: might fail for the first columns...
                if row_cells[index_not_none_key].value is None:
                    continue
            key_values_list.append({
                key: row_cells[column_ID_int]
                for key, column_ID_int in key_column_ID_int_dict.items()
            })
        except IndexError as e:  # todo: not really clean
            continue
    return key_values_list

def open_xls_input(filename, read_only):
    return openpyxl.load_workbook(filename=filename, read_only=read_only,
                                  keep_vba=True)
